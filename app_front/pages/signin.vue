<script setup lang="ts">
import {validateEmail} from "~/composables/validators";
import type {ApiError} from "~/composables/fetchApi";

definePageMeta({
  layout: 'noscroll',
})
useHead({
  title: 'Sign in',
})

const email = ref('')
const email_small = ref('')
const password = ref('')
const error = ref('')

const onSubmit = () => {
  let validated: boolean = validateEmail(email.value)
  if (!validated) {
    email_small.value = 'Invalid email'
  } else {
    email_small.value = ''
    error.value = ''
    useUserStore().signIn(email.value, password.value)
        .catch((e: ApiError) => {
          error.value = e.message
        })
        .then(() => {
          console.log("Signed in")
        })
  }
}
</script>
<template>
  <main>
    <h1>Sign in to Archypix</h1>
    <form>
      <InputInForm name="Email" type="email" aria="Email" v-model:value="email" v-model:small="email_small"
                   small_error/>
      <InputInForm name="Password" type="password" aria="Password" v-model:value="password"
                   link_url="/resetpassword" link_name="Forgot password?"/>
      <Button label="Sign in" icon="pi pi-sign-in" @click="onSubmit"/>
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
