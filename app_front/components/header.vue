<script setup lang="ts">
const isStandalone = useRuntimeConfig()?.public?.isStandalone

let user = useUserStore()
let isLoggedIn = user.isLoggedIn(false, true);

</script>

<template>
  <header>
    <div class="left">
      <div class="icon">
        <img src="/favicon.ico" alt="Archypix icon">
      </div>
      <h2>
        Archypix
        <span v-if="isStandalone">Standalone</span>
      </h2>
    </div>
    <div class="right">
      <nav>
        <ul>
          <template v-if="!isLoggedIn">
            <li>
              <nuxt-link to="/signin"><span class="pi pi-sign-in"/>Sign in</nuxt-link>
            </li>
            <li>
              <nuxt-link to="/signup"><span class="pi pi-user-plus"/>Sign up</nuxt-link>
            </li>
          </template>
          <template v-else>
            <li>
              <nuxt-link to="/"><span class="pi pi-images"/>Archypix webapp</nuxt-link>
            </li>
          </template>
          <template v-if="user.isAdmin">
            <li>
              <nuxt-link to="/admin"><span class="pi pi-lock"/>Admin</nuxt-link>
            </li>
          </template>

        </ul>
      </nav>
    </div>
  </header>
</template>

<style scoped lang="stylus">
header
  height 48px
  position fixed
  -webkit-backdrop-filter: blur(8px);
  backdrop-filter: blur(8px);
  background-color: hsla(174, 53%, 90%, 0.7);
  border-bottom: 1px solid hsl(174, 53%, 90%);

  z-index 1
  width 100vw

  display flex
  justify-content space-between
  align-items stretch

  padding 0 1rem
  @media screen and (min-width: 600px)
    padding 0 3rem
  @media screen and (min-width: 800px)
    padding 0 5rem

  h2 span
    font-size .8rem
    color var(--primary-800)

.left, .right
  display flex
  align-items center

.left
  gap .5rem

nav
  height 100%

  ul
    list-style none
    height 100%
    padding 0
    margin 0
    display flex
    flex-wrap wrap
    gap .7rem


    li
      height 100%
      display block
      position relative

      &.current a
        border-color hsl(174, 90%, 40%)

      a
        position relative
        top 50%
        transform translateY(-50%)
        display flex
        align-items center
        white-space nowrap
        gap .5rem

        font-size 1.1rem;
        vertical-align center
        padding .4rem .7rem
        text-decoration none
        color var(--text-color)

        border-radius .5rem
        background-color var(--surface-ground)
        border 1px solid var(--surface-border)

        &:hover
          background-color var(--surface-hover)
</style>
