import type {Ref} from "vue";
import type {ApiError} from "~/composables/fetchApi";

export enum UserStatus {
    Unconfirmed = 'Unconfirmed',
    Normal = 'Normal',
    Banned = 'Banned',
    Admin = 'Admin',
    NotConnected = 'NotConnected',
    Unknown = 'Unknown',
    AwaitingSignInCode = 'AwaitingSignInCode',
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
    email: string
    status: UserStatus
}
export type SignInEmailResponse = {
    user_id: string
    code_token: string
}
export type SignUpResponse = {
    user_id: string
    code_token: string
}

export type ConfirmSignInUpResponse = {
    user_id: string
    auth_token: string
    name: string
    email: string
    status: UserStatus
    redirect_url: String
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
    const isAdmin = computed(() => {
        return status.value == UserStatus.Admin
    })
    const isAwaitingSignInCode = computed(() => {
        return status.value == UserStatus.AwaitingSignInCode
    })

    const signInWithEmail2FA = async (user_email: string, password: string) => {
        let redirect_url = useRoute().query?.r?.toString()
        return useFetchApi(false, 'POST', null, null, '/auth/signin/email', {email: user_email, password, redirect_url})
            // @ts-ignore cause ts wants type void | SignInResponse but it's SignInResponse
            .then((data: SignInEmailResponse) => {
                id.value = data.user_id
                setConfirmCodeToken(ConfirmAction.Signin, data.code_token, 15)
                status.value = UserStatus.AwaitingSignInCode
                return data
            })
    }
    const signIn = async (user_email: string, password: string, totp_code: string | undefined = undefined) => {
        if (!totp_code) totp_code = undefined
        return useFetchApi(false, 'POST', null, null, '/auth/signin', {email: user_email, password, totp_code})
            // @ts-ignore cause ts wants type void | SignInResponse but it's SignInResponse
            .then((data: SignInResponse) => signInFromData(data))

    }
    const signInFromData = (data: SignInResponse | ConfirmSignInUpResponse): SignInResponse | ConfirmSignInUpResponse => {
        email.value = data.email
        status.value = data.status
        name.value = data.name
        id.value = data.user_id
        auth_token.value = data.auth_token
        return data
    }
    const signUp = async (name: string, email: string, password: string) => {
        let redirect_url = useRoute().query?.r?.toString()
        return useFetchApi(false, 'POST', null, null, '/auth/signup', {name, email, password, redirect_url})
            // @ts-ignore cause ts wants type void | SignUpResponse but it's SignUpResponse
            .then((data: SignUpResponse) => {
                id.value = data.user_id
                setConfirmCodeToken(ConfirmAction.Signup, data.code_token, 15)
                status.value = UserStatus.Unconfirmed
                return data
            })
    }

    const updateStatus = async () => {
        // id = useCookie('px_user_id')
        // auth_token = useCookie('px_auth_token')
        if (id.value && auth_token.value) {
            await useGetApi(true, '/auth/status')
                // @ts-ignore cause ts wants type void | AuthStatus but it's AuthStatus
                .then((data: AuthStatus) => {
                    status.value = data.status
                    name.value = data.name
                    email.value = data.email
                })
                .catch((error: ApiError | null) => {
                    if (error && error.error_type === ErrorType.Unauthorized) {
                        status.value = UserStatus.NotConnected
                        id.value = null
                        auth_token.value = null
                    } else {
                        status.value = UserStatus.Unknown
                    }
                })
        } else {
            status.value = UserStatus.NotConnected
        }
        if (status.value == UserStatus.NotConnected) {
            if (id.value && getConfirmCodeToken(ConfirmAction.Signup)) {
                status.value = UserStatus.Unconfirmed
            } else if (id.value && getConfirmCodeToken(ConfirmAction.Signin)) {
                status.value = UserStatus.AwaitingSignInCode
            }
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
            message: 'No confirm code token found, please try again'
        } as ApiError)
        return useFetchApi(false, 'POST', auth_token.value, id.value, '/auth/confirm/code', {action, code, code_token})
            .then((data) => {
                removeConfirmToken(action)
                return data
            })
    }
    const confirmWithToken = async (action: ConfirmAction, token: string) => {
        return useFetchApi(true, 'POST', auth_token.value, id.value, '/auth/confirm/token', {action, token})
            .then((data) => {
                removeConfirmToken(action)
                return data
            })
    }


    return {
        status, name, email, id, auth_token,
        isLoggedIn, isUnconfirmed, isAwaitingSignInCode, isAdmin, signIn, signInWithEmail2FA, signInFromData, signUp,
        updateStatus,
        getConfirmCodeToken, setConfirmCodeToken, removeConfirmToken, confirmWithCode, confirmWithToken
    }
})
