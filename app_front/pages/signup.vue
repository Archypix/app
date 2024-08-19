<script setup lang="ts">
import {validateEmail, validatePasswordConfirmation, validateUserName} from "~/composables/validators";
import type {ApiError} from "~/composables/fetchApi";
import type {SignUpResponse} from "~/stores/user";

definePageMeta({
  layout: 'noscroll',
})
useHead({
  title: 'Sign up',
})

const name = ref('')
const name_small = ref('')
const email = ref('')
const email_small = ref('')
const password = ref('')
const password_small = ref('')
const password_confirm = ref('')
const password_confirm_small = ref('')
const password_visible = ref(false)

const error = ref('')

watch(password, () => {
  if (password_confirm_small && !validatePasswordConfirmation(password.value, password_confirm.value)) password_confirm_small.value = ''
})
watch(password_confirm, () => {
  if (password_confirm_small && !validatePasswordConfirmation(password.value, password_confirm.value)) password_confirm_small.value = ''
})

const onSubmit = () => {
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
    useUserStore().signUp(name.value, email.value, password.value)
        .catch((e: ApiError) => {
          error.value = e.message
        })
        .then((data: SignUpResponse | void) => {
          console.log("Signed up", data)
        })
  }
}
</script>
<template>
  <main>
    <h1>Sign up to Archypix</h1>
    <form>
      <InputInForm name="Full name" type="name" v-model:value="name" v-model:small="name_small"
                   small_error default_focus/>
      <InputInForm name="Email" type="email" v-model:value="email" v-model:small="email_small"
                   small_error/>
      <InputInForm name="Password" type="password" v-model:value="password"
                   v-model:small="password_small" v-model:password_visible="password_visible" small_error/>
      <InputInForm name="Confirm password" type="password" v-model:value="password_confirm"
                   v-model:small="password_confirm_small" v-model:password_visible="password_visible"
                   small_error disable_error_auto_remove/>

      <Button label="Sign up" icon="pi pi-user-plus" @click="onSubmit"
              :disabled="name_small != '' || email_small != '' || password_small != '' || password_confirm_small != ''"/>
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
