<script setup lang="ts">
import {validateEmail} from "~/composables/validators";
import type {ApiError} from "~/composables/fetchApi";
import {type SignInResponse, useUserStore} from "~/stores/user";

definePageMeta({
  layout: 'noscroll'
})
useHead({
  title: 'Sign in',
})

const user = useUserStore()

const error = ref('')
const loading = ref(false)

const email = ref('')
const email_small = ref('')
const password = ref('')
const password_visible = ref(false)

const onSubmitSignin = () => {
  error.value = ''

  let email_error = validateEmail(email.value)
  if (email_error) {
    email_small.value = email_error
  } else {
    email_small.value = ''
    loading.value = true
    useUserStore().signIn(email.value, password.value)
        .then((data: SignInResponse) => {

        })
        .catch((e: ApiError) => {
          loading.value = false
          error.value = e.message
        })
  }
}
</script>
<template>
  <main>
    <h1>Sign in to Archypix</h1>

    <div v-if="loading" class="loading">
      <ProgressSpinner style="width: 50px; height: 50px" strokeWidth="8" fill="transparent"/>
      <p>Loading</p>
    </div>
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
