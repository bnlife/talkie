<script setup lang="ts">
import type { SelectTriggerProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import type { SelectTriggerVariants } from "."
import { reactiveOmit } from "@vueuse/core"
import { ChevronDown } from "lucide-vue-next"
import { SelectIcon, SelectTrigger, useForwardProps } from "reka-ui"
import { cn } from "@/lib/utils"
import { selectTriggerVariants } from "."

const props = defineProps<SelectTriggerProps & {
  class?: HTMLAttributes["class"]
  variant?: SelectTriggerVariants["variant"]
  size?: SelectTriggerVariants["size"]
}>()

const delegatedProps = reactiveOmit(props, ["class", "variant", "size"])

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectTrigger
    v-bind="forwardedProps"
    :class="cn(selectTriggerVariants({ variant, size }), props.class)"
  >
    <slot />
    <SelectIcon as-child>
      <ChevronDown class="w-4 h-4 opacity-50 shrink-0" />
    </SelectIcon>
  </SelectTrigger>
</template>
