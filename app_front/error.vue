<script setup lang="ts">
import type {NuxtError} from "#app";

const props = defineProps({
  error: Object as () => NuxtError
})

let user = useUserStore()
await user.updateStatus()

const handleError = () => clearError({redirect: '/'})

</script>

<template>
  <NuxtLayout name="noscroll">
    <main>

      <template v-if="props?.error?.statusCode === 404">
        <h1>404 - Page Not Found</h1>
        <p>The page you are looking for does not exist.</p>
      </template>
      <template v-else>
        <h1>Error {{props?.error?.statusCode}}</h1>
        <p>Please contact administrators</p>
      </template>

      <Button @click="handleError" label="Go Home" severity="secondary" outlined></Button>
    </main>
  </NuxtLayout>
</template>

<style scoped lang="stylus">
main
  display flex
  flex-direction column
  align-items center
  h1
    color var(--red-600)
  p
    margin-bottom 20px

</style>
