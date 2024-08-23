import {useUserStore} from "~/stores/user";
import {hash} from "ohash";


export enum ErrorType {
    BadRequest = 'BadRequest',
    Unauthorized = 'Unauthorized',
    NotFound = 'NotFound',
    UnprocessableEntity = 'UnprocessableEntity',
    InternalError = 'InternalError',
    // Form validation (see UnprocessableEntity for type check related errors)
    InvalidInput = 'InvalidInput',
    // Sign in / status types
    UserNotFound = 'UserNotFound',
    UserBanned = 'UserBanned',
    UserUnconfirmed = 'UserUnconfirmed',
    // Sign in types
    InvalidEmailOrPassword = "InvalidEmailOrPassword",
    TFARequiredOverEmail = "TFARequiredOverEmail",
    TFARequired = "TFARequired",
    InvalidTOTPCode = "InvalidTOTPCode",
    // Sign up types
    EmailAlreadyExists = "EmailAlreadyExists",
    // Confirm
    ConfirmationAlreadyUsed = "ConfirmationAlreadyUsed",
    ConfirmationNotFound = "ConfirmationNotFound",
    // Admin
    UserNotAdmin = 'UserNotAdmin',
    // Database error
    DatabaseError = 'DatabaseError',
    // Front end
    NoConfirmCodeToken = 'NoConfirmCodeToken',
    CantReachApi = 'CantReachApi',

}

export type ApiError = {
    error_type: ErrorType
    message: string
}
export type HttpError = {
    statusCode: number
    statusMessage: string
    data: ApiError | null
}


export const useGetApi = async function <R>(ssr: boolean = false, path: string) {

    let user = useUserStore()
    return await useFetchApi<undefined, R>(ssr, 'GET', user.auth_token, user.id, path, undefined)
}

export const usePostApi = async function <B, R>(ssr: boolean = false, path: string, data: any) {

    let user = useUserStore()
    return await useFetchApi<B, R>(ssr, 'POST', user.auth_token, user.id, path, data)
}

export const useFetchApi = async function <B, R>(ssr: boolean = false, method: string, auth_token: string | null | undefined,
                                                 id: string | null | undefined, path: string, body: B): Promise<R> {

    return new Promise<R>(async (resolve: (data: R) => void, reject: (error: ApiError | null) => void) => {
        const backend_host = import.meta.server ? useRuntimeConfig()?.public?.backendHostSSR : useRuntimeConfig()?.public?.backendHost;
        console.log('useFetchApi', 'ssr:', ssr, 'method:', method, 'id:', id, 'path:', path, 'body:', body)

        // @ts-ignore
        let {data, error} = await useFetch<R, HttpError>(backend_host + path, {
            key: hash([ssr, method, auth_token, id, path, body]),
            method: method,
            headers: {
                'User-Agent': import.meta.server ? '' : window.navigator.userAgent, // Needed only for login & signup queries.
                'X-Auth-Token': auth_token,
                'X-User-Id': id
            },
            server: ssr,
            body: body,
        })
        if (data.value) {
            console.log('useFetchApi', 'Success:', data.value)
            resolve(data.value as R)
        } else {
            let error_data = error.value?.data ?? null;
            if (error_data == null) {
                console.error('useFetchApi', 'Unknown error:', error.value?.statusCode, error.value?.statusMessage, error)
                error_data = {
                    error_type: ErrorType.CantReachApi,
                    message: 'Unable to reach the API, please try again later'
                }
            } else {
                console.log('useFetchApi', 'Known error:', error_data.error_type, '-', error_data.message)
            }
            reject(error_data)
        }
    })
}
