<script setup lang="ts">

</script>

<template>
  <div>
    <Header/>
    <div class="main-content">
      <slot/>
    </div>
    <Footer/>
  </div>
</template>

<style lang="stylus" src="@/assets/css/default.styl"/>

<style scoped lang="stylus">

.main-content
  background-color var(--primary-50)
  flex-grow 1
  padding-top 48px

  display flex
  justify-content center
  align-items stretch

  @media screen and (min-width: 400px)
    align-items center

:deep(p), :deep(h1), :deep(h2), :deep(h3), :deep(h4), :deep(h5), :deep(h6)
  text-align center

:deep(h1)
  font-size 1.5em
  margin-bottom 1em

:deep(main)
  background-color white;
  padding 10px
  padding-top 40px
  flex-basis 100%;

  @media screen and (min-width: 400px)
    border-radius 10px
    margin: 20px 0;
    padding 20px
    height fit-content
    flex-basis 350px
    border: 1px solid var(--primary-100)
    filter: drop-shadow(0 0 10px var(--primary-100))


  form
    display flex
    flex-direction column
    align-items center
    gap 10px

    .input-in-form
      width 100%
      max-width 250px

      small
        margin-top: 2px


    button[type="submit"]
      width 100%
      max-width 150px
      margin 1em 0

    button.p-button-link
      padding 0

  div.loading
    display flex
    flex-direction column
    justify-content center
    align-items center
    gap 10px
    margin 40px
    font-size 1.2em


</style>
