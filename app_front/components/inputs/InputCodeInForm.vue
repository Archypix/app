<script setup lang="ts">

const props = defineProps({
  name: String,
  value: String,
  small: String,
  small_error: Boolean,
  disable_error_auto_remove: Boolean,
  default_focus: Boolean,
  length: {
    type: Number,
    default: 4
  }
})

const emit = defineEmits(['update:value', 'update:small', 'update:password_visible'])
const local_value = ref(props.value)
watch(local_value, () => emit('update:value', local_value.value))

let id = computed(() => props.name ? props.name.toLowerCase().replaceAll(' ', '-') + '-input' : '')

function onInput(e: Event) {
  emit('update:value', (e.target as HTMLInputElement).value)
  if (props.small_error && !props.disable_error_auto_remove) {
    emit('update:small', '') // Clear small error when input is changed
  }
}

const target = ref()
onMounted(() => {
  if (props.default_focus) {
    nextTick(() => {
      target?.value?.querySelector('input')?.focus();
    });
  }
})

</script>

<template>
  <div class="inputcode-in-form" ref="target">
    <div class="header" v-if="name">
      <label :for="id">{{ name }}</label>
    </div>

    <InputOtp
        :id="id"
        :length="props.length"
        v-model="local_value"
        :aria-labelledby="name"
        :invalid="small_error && small?.length != 0"
        integerOnly/>

    <small v-if="props.small"
           :style="props.small_error ? 'color: var(--red-700);' : ''">
      {{ props.small }}
    </small>
  </div>
</template>

<style scoped lang="stylus">
.inputcode-in-form

  .header
    display flex
    justify-content space-around
    margin-bottom 5px

  input
    width 100%

</style>
