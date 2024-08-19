import type {Ref} from "vue";
import type {ApiError} from "~/composables/fetchApi";

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
    status: UserStatus
    name: string
    id: string
    auth_token: string
}

export type SignUpResponse = {
    id: string
    code_token: string
}


export const useUserStore = defineStore('user', () => {

    // Data
    const status = ref(UserStatus.Unknown)
    const name: Ref<string | null> = ref(null)
    const email: Ref<string | null> = ref(null)
    let id = useCookie('px_user_id', {watch: true})
    let auth_token = useCookie('px_auth_token', {watch: true})
    const code_token = ref('')

    // Methods
    const isLoggedIn = (accept_unconfirmed: boolean = false, accept_banned: boolean = false) => {
        return status.value == UserStatus.Normal
            || (accept_unconfirmed && status.value == UserStatus.Unconfirmed)
            || (accept_banned && status.value == UserStatus.Banned)
    }
    const isUnconfirmed = () => {
        return status.value == UserStatus.Unconfirmed
    }
    const isAdmin = () => {
        return status.value == UserStatus.Admin
    }
    const signIn = async (email: string, password: string) => {

        return useFetchApi(false, 'POST', null, null, '/auth/signin', {email, password})
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
                status.value = data.status
                name.value = data.name
                id.value = data.id
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
                code_token.value = data.code_token
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
        } else {
            status.value = UserStatus.NotConnected
        }
    }


    return {
        status, name, email, id, auth_token, code_token,
        isLoggedIn, isUnconfirmed, isAdmin, signIn,
        updateStatus
    }
})
