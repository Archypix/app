<script setup lang="ts">

const props = defineProps({
  name: String,
  value: String,
  type: String,
  aria: String,
  icon: String,
  small: String,
  small_error: Boolean,
  link_url: String,
  link_name: String
})

const target = ref()
const emit = defineEmits(['update:value', 'update:small'])

let id: string = '';
if(props.name){
  id = props.name.toLowerCase() + '-input';
}

function onInput(e: Event) {
  emit('update:value', (e.target as HTMLInputElement).value)
  if (props.small_error) {
    emit('update:small', '') // Clear small error when input is changed
  }
}

</script>

<template>
  <div class="input-in-form">
    <div class="header" v-if="name">
      <label :for="id">{{ name }}</label>
      <label v-if="link_url && link_name" :for="id">
        <nuxt-link :href="link_url">{{ link_name }}</nuxt-link>
      </label>
    </div>

    <InputText
        :id="id"
        :ref="target"
        :type="type"
        :value="props.value"
        @input="onInput($event)"
        :aria-describedby="props.aria"
        :invalid="small_error && small?.length != 0"
        autocomplete="on"/>

    <small v-if="props.small"
           :style="props.small_error ? 'color: var(--red-700);' : ''">
      {{ props.small }}
    </small>
  </div>
</template>

<style scoped lang="stylus">
.input-in-form
  *
    display block
  .header
    display flex
    justify-content space-between
    margin-bottom 5px

  input
    width 100%

</style>
