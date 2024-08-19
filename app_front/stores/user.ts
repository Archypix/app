import type {Ref} from "vue";
import type {ApiError} from "~/composables/fetchApi";
import {super} from "@babel/types";

export enum UserStatus {
    Unconfirmed = 'Unconfirmed',
    Normal = 'Normal',
    Banned = 'Banned',
    Admin = 'Admin',
    NotConnected = 'NotConnected',
    Unknown = 'Unknown'
}

export type AuthStatus = {
    status: UserStatus
    name: string
    email: string
}

export type SignInResponse = {
    user_id: string
    auth_token: string
    name: string
    status: UserStatus
}

export type SignUpResponse = {
    id: string
    code_token: string
}

export type ConfirmResponse = {
    auth_token: string | null
}

export enum ConfirmAction {
    Signup = "Signup",
    Signin = "Signin",
    ResetPassword = "ResetPassword",
    DeleteAccount = "DeleteAccount"
}

export const useUserStore = defineStore('user', () => {

    // Data
    const status = ref(UserStatus.Unknown)
    const name: Ref<string | null> = ref(null)
    const email: Ref<string | null> = ref(null)
    let id = useCookie('px_user_id', {watch: true})
    let auth_token = useCookie('px_auth_token', {watch: true})

    // Methods
    const isLoggedIn = (accept_unconfirmed: boolean = false, accept_banned: boolean = false) => {
        return status.value == UserStatus.Normal
            || (accept_unconfirmed && status.value == UserStatus.Unconfirmed)
            || (accept_banned && status.value == UserStatus.Banned)
    }
    const isUnconfirmed = computed(() => {
        return status.value == UserStatus.Unconfirmed
    })
    const isAdmin = () => {
        return status.value == UserStatus.Admin
    }
    const signIn = async (user_email: string, password: string) => {

        return useFetchApi(false, 'POST', null, null, '/auth/signin', {user_email, password})
            .catch((error: ApiError | null) => {
                if (error?.error_type === ErrorType.Unauthorized) {
                    status.value = UserStatus.NotConnected
                } else {
                    status.value = UserStatus.Unknown
                }
                throw error
            })
            // @ts-ignore cause ts wants type void | SignInResponse but it's SignInResponse
            .then((data: SignInResponse) => {
                email.value = user_email
                status.value = data.status
                name.value = data.name
                id.value = data.user_id
                auth_token.value = data.auth_token
                return data
            })
    }
    const signUp = async (name: string, email: string, password: string) => {
        return useFetchApi(false, 'POST', null, null, '/auth/signup', {name, email, password})
            .catch((error: ApiError | null) => {
                if (error?.error_type === ErrorType.Unauthorized) {
                    status.value = UserStatus.NotConnected
                } else {
                    status.value = UserStatus.Unknown
                }
                throw error
            })
            // @ts-ignore cause ts wants type void | SignUpResponse but it's SignUpResponse
            .then((data: SignUpResponse) => {
                status.value = UserStatus.Unconfirmed
                id.value = data.id
                setConfirmCodeToken(ConfirmAction.Signup, data.code_token, 15)
                return data
            })
    }

    const updateStatus = async () => {
        // id = useCookie('px_user_id')
        // auth_token = useCookie('px_auth_token')
        if (id.value && auth_token.value) {
            await useGetApi(true, '/auth/status')
                .catch((error: ApiError | null) => {
                    if (error && error.error_type === ErrorType.Unauthorized) {
                        status.value = UserStatus.NotConnected
                    } else {
                        status.value = UserStatus.Unknown
                    }
                    // id.value = null
                    // auth_token.value = null
                })
                // @ts-ignore cause ts wants type void | AuthStatus but it's AuthStatus
                .then((data: AuthStatus) => {
                    status.value = data.status
                    name.value = data.name
                    email.value = data.email
                })
        } else if (id.value && getConfirmCodeToken(ConfirmAction.Signup)) {
            status.value = UserStatus.Unconfirmed
        } else {
            status.value = UserStatus.NotConnected
        }
    }

    // Confirm tokens
    const getConfirmCodeToken = (action: ConfirmAction) => {
        return useCookie('px_confirm_' + action.toLowerCase() + '_code_token').value
    }
    const setConfirmCodeToken = (action: ConfirmAction, token: string, expiry_min: number) => {
        let options = {maxAge: expiry_min * 60}
        useCookie('px_confirm_' + action.toLowerCase() + '_code_token', options).value = token
    }
    const removeConfirmToken = (action: ConfirmAction) => {
        useCookie('px_confirm_' + action.toLowerCase() + '_code_token').value = null
    }
    const confirmWithCode = async (action: ConfirmAction, code: number) => {
        const code_token = getConfirmCodeToken(action)
        if (!code_token) return Promise.reject({
            error_type: ErrorType.NoConfirmCodeToken,
            message: 'No confirm code token'
        } as ApiError)
        return useFetchApi(false, 'POST', auth_token.value, id.value, '/auth/confirm/code', {action, code, code_token})
    }


    return {
        status, name, email, id, auth_token,
        isLoggedIn, isUnconfirmed, isAdmin, signIn, signUp,
        updateStatus,
        getConfirmCodeToken, setConfirmCodeToken, removeConfirmToken, confirmWithCode
    }
})
