<script setup lang="ts">
import {validateEmail} from "~/composables/validators";
import type {ApiError} from "~/composables/fetchApi";
import {ConfirmAction, type ConfirmSignInUpResponse, UserStatus, useUserStore} from "~/stores/user";
import InputCodeInForm from "~/components/inputs/InputCodeInForm.vue";

definePageMeta({
  layout: 'noscroll'
})
useHead({
  title: 'Sign in',
})

const user = useUserStore()
if (user.isLoggedIn()) {
  useRouter().push('/')
}


const error = ref('')
const loading = ref(false)

const email = ref('')
const email_small = ref('')
const password = ref('')
const password_visible = ref(false)

const totp_step = ref(false)

const code = ref('')
const code_small = ref('')

const catchError = (e: ApiError) => {
  loading.value = false
  error.value = e.message
}

// Confirm from link /signin?id=...&token=...
const route = useRoute()
if (route.query.id && route.query.token) {
  loading.value = true
  user.id = route.query.id.toString()
  user.confirmWithToken(ConfirmAction.Signin, route.query.token.toString())
      // @ts-ignore
      .then((data: ConfirmSignInUpResponse) => {
        user.signInFromData(data)
        navigateTo(data.redirect_url as string, {external: true})
      }).catch(catchError)
}

const onSubmitSignin = () => {
  error.value = ''

  let email_error = validateEmail(email.value)
  if (email_error) {
    email_small.value = email_error
    return
  }
  email_small.value = ''
  loading.value = true
  password_visible.value = false
  user.signIn(email.value, password.value, code.value)
      .then(() => {
        console.log(useRoute().query)
        navigateTo(useRoute().query?.r?.toString() ?? '/', {external: true})
      })
      .catch((e: ApiError) => {
        if (e.error_type == ErrorType.TFARequiredOverEmail) {
          onSendConfirmationCodeByEmail()
        } else if (e.error_type == ErrorType.TFARequired) {
          loading.value = false
          totp_step.value = true
        } else catchError(e)
      })
}
const onSubmitConfirmCode = () => {
  error.value = ''
  loading.value = true
  user.confirmWithCode(ConfirmAction.Signin, parseInt(code.value, 10))
      // @ts-ignore
      .then((data: ConfirmSignInUpResponse) => {
        user.signInFromData(data)
        navigateTo(data.redirect_url as string, {external: true})
      }).catch(catchError)
}
const onSendConfirmationCodeByEmail = () => {
  error.value = ''
  loading.value = true
  totp_step.value = false
  user.signInWithEmail2FA(email.value, password.value)
      .then(() => {
        loading.value = false
      }).catch(catchError)
}

const onCancelConfirm = () => {
  totp_step.value = false
  user.removeConfirmToken(ConfirmAction.Signin)
  user.status = UserStatus.NotConnected
  error.value = ''
  code.value = ''
  code_small.value = ''
}
const onSwitchToEmail2FA = () => {
  onSendConfirmationCodeByEmail()
}

</script>
<template>
  <main>
    <h1>Sign in to Archypix</h1>

    <div v-if="loading" class="loading">
      <ProgressSpinner style="width: 50px; height: 50px" strokeWidth="8" fill="transparent"/>
      <p>Loading</p>
    </div>
    <form @submit.prevent="onSubmitSignin" v-else-if="totp_step">
      <p>Enter the TOTP code</p>

      <InputCodeInForm name="Code" :length="6" v-model:value="code" v-model:small="code_small" small_error
                       default_focus/>

      <Button label="Cancel" @click="onCancelConfirm" link/>
      <Button label="Send me a code by email instead" @click="onSwitchToEmail2FA" link/>
      <Button label="Sign in" icon="pi pi-sign-in" type="submit" :disabled="code_small != ''"/>
    </form>
    <form @submit.prevent="onSubmitConfirmCode" v-else-if="user.isAwaitingSignInCode">
      <p>Enter the 4-digit code received by email<br>or follow the received link.</p>

      <InputCodeInForm name="Code" v-model:value="code" v-model:small="code_small" small_error default_focus/>

      <Button label="Cancel" @click="onCancelConfirm" link/>
      <Button label="Sign in" icon="pi pi-sign-in" type="submit" :disabled="code_small != ''"/>
    </form>
    <form @submit.prevent="onSubmitSignin" v-else>
      <InputInForm name="Email" type="email" aria="Email" v-model:value="email" v-model:small="email_small"
                   small_error ref="first_input" default_focus/>
      <InputInForm name="Password" type="password" aria="Password" v-model:value="password"
                   v-model:password_visible="password_visible" link_url="/resetpassword"
                   link_name="Forgot password?"/>
      <Button label="Sign in" icon="pi pi-sign-in" type="submit" :disabled="email_small != ''"/>
    </form>


    <Message severity="error" icon="pi pi-info-circle" v-if="error">
      <span>{{ error }}</span>
    </Message>

    <p>Don't have an account?
      <nuxt-link to="/signup">Sign up</nuxt-link>
    </p>
  </main>
</template>

<style scoped lang="stylus">

</style>
