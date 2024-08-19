<script setup lang="ts">

const props = defineProps({
  name: String,
  value: String,
  type: String,
  icon: String,
  small: String,
  small_error: Boolean,
  link_url: String,
  link_name: String,
  password_visible: {
    type: Boolean,
    default: undefined
  },
  disable_error_auto_remove: Boolean,
  default_focus: Boolean,
})

const emit = defineEmits(['update:value', 'update:small', 'update:password_visible'])

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
  <div class="input-in-form" ref="target">
    <div class="header" v-if="name">
      <label :for="id">{{ name }}</label>
      <label v-if="link_url && link_name" :for="id">
        <nuxt-link :href="link_url">{{ link_name }}</nuxt-link>
      </label>
    </div>

    <InputGroup>
      <InputGroupAddon v-if="icon">
        <i class="{{icon}}"></i>
      </InputGroupAddon>
      <InputText
          :id="id"
          :type="password_visible ? 'text' : type"
          :value="value"
          @input="onInput($event)"
          :aria-labelledby="name"
          :invalid="small_error && small?.length != 0"
          autocomplete="on"/>
      <Button v-if="type == 'password' && password_visible !== undefined"
              :icon="password_visible ? 'pi pi-eye-slash' : 'pi pi-eye'" severity="contrast"
              @click="emit('update:password_visible', !password_visible)"/>
    </InputGroup>

    <small v-if="props.small"
           :style="props.small_error ? 'color: var(--red-700);' : ''">
      {{ props.small }}
    </small>
  </div>
</template>

<style scoped lang="stylus">
.input-in-form

  .header
    display flex
    justify-content space-between
    margin-bottom 5px

  input
    width 100%

</style>
