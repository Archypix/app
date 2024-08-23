<script setup lang="ts">
import {validateEmail, validatePasswordConfirmation, validateUserName} from "~/composables/validators";
import {type ApiError} from "~/composables/fetchApi";
import {ConfirmAction, type ConfirmSignInUpResponse, useUserStore} from "~/stores/user";
import InputCodeInForm from "~/components/inputs/InputCodeInForm.vue";

definePageMeta({
  layout: 'noscroll',
})
useHead({
  title: 'Sign up',
})

const user = useUserStore()
if (user.isLoggedIn()) {
  useRouter().push('/')
}

const error = ref('')
const loading = ref(false)

const name = ref('')
const name_small = ref('')
const email = ref('')
const email_small = ref('')
const password = ref('')
const password_small = ref('')
const password_confirm = ref('')
const password_confirm_small = ref('')
const password_visible = ref(false)

watch(password, () => {
  if (password_confirm_small && !validatePasswordConfirmation(password.value, password_confirm.value)) password_confirm_small.value = ''
})
watch(password_confirm, () => {
  if (password_confirm_small && !validatePasswordConfirmation(password.value, password_confirm.value)) password_confirm_small.value = ''
})

const catchError = (e: ApiError) => {
  loading.value = false
  error.value = e.message
}

// Confirm from link /signup?id=...&token=...
const route = useRoute()
if (route.query.id && route.query.token) {
  loading.value = true
  user.id = route.query.id.toString()
  user.confirmWithToken(ConfirmAction.Signup, route.query.token.toString())
      // @ts-ignore
      .then((data: ConfirmSignInUpResponse) => {
        user.signInFromData(data)
        navigateTo(data.redirect_url as string, {external: true})
      }).catch(catchError)
}

const onSubmitSignup = () => {
  error.value = ''

  let name_error = validateUserName(name.value)
  if (name_error) name_small.value = name_error
  else name_small.value = ''

  let email_error = validateEmail(email.value)
  if (email_error) email_small.value = email_error
  else email_small.value = ''

  let password_error = validatePassword(password.value)
  if (password_error) password_small.value = password_error
  else password_small.value = ''

  let password_match_error = validatePasswordConfirmation(password.value, password_confirm.value)
  if (password_match_error) password_confirm_small.value = password_match_error
  else password_confirm_small.value = ''

  if (!name_error && !email_error && !password_error && !password_match_error) {
    loading.value = true
    user.signUp(name.value, email.value, password.value)
        .then(() => {
          loading.value = false
          password_visible.value = false
        }).catch(catchError)
  }
}

const code = ref('')
const code_small = ref('')

const onSubmitConfirm = () => {
  error.value = ''
  loading.value = true

  console.log(code.value)
  if (code.value.length !== 4) {
    code_small.value = 'Code must be 4 digits'
    loading.value = false
    return
  }

  user.confirmWithCode(ConfirmAction.Signup, parseInt(code.value, 10))
      // @ts-ignore
      .then((data: ConfirmSignInUpResponse) => {
        user.signInFromData(data)
        navigateTo(data.redirect_url as string, {external: true})
      }).catch(catchError)
}

const onCancelConfirm = () => {
  user.removeConfirmToken(ConfirmAction.Signup)
  nextTick(() => {
    code.value = ''
    code_small.value = ''
    user.updateStatus()
  })
}

</script>
<template>
  <main>
    <h1>Sign up to Archypix</h1>

    <div v-if="loading" class="loading">
      <ProgressSpinner style="width: 50px; height: 50px" strokeWidth="8" fill="transparent"/>
      <p>Loading</p>
    </div>
    <form @submit.prevent="onSubmitSignup" v-else-if="!user.isUnconfirmed">
      <InputInForm name="Full name" type="name" v-model:value="name" v-model:small="name_small"
                   small_error default_focus/>
      <InputInForm name="Email" type="email" v-model:value="email" v-model:small="email_small"
                   small_error/>
      <InputInForm name="Password" type="password" v-model:value="password"
                   v-model:small="password_small" v-model:password_visible="password_visible" small_error/>
      <InputInForm name="Confirm password" type="password" v-model:value="password_confirm"
                   v-model:small="password_confirm_small" v-model:password_visible="password_visible"
                   small_error disable_error_auto_remove/>

      <Button label="Sign up" icon="pi pi-user-plus" type="submit"
              :disabled="name_small != '' || email_small != '' || password_small != '' || password_confirm_small != ''"/>
    </form>
    <form @submit.prevent="onSubmitConfirm" v-else>
      <p>Enter the 4-digit code received by email<br>or follow the received link.</p>

      <InputCodeInForm name="Code" v-model:value="code" v-model:small="code_small" small_error default_focus/>

      <Button label="Cancel" @click="onCancelConfirm" link/>
      <Button label="Sign up" icon="pi pi-user-plus" type="submit" :disabled="code_small != ''"/>
    </form>

    <Message severity="error" icon="pi pi-info-circle" v-if="error">
      <span>{{ error }}</span>
    </Message>

    <p>Already have an account?
      <nuxt-link to="/signin">Sign in</nuxt-link>
    </p>
  </main>
</template>

<style scoped lang="stylus">

</style>
