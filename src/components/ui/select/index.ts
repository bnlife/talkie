import type { VariantProps } from "class-variance-authority"
import { cva } from "class-variance-authority"

export { default as Select } from "./Select.vue"
export { default as SelectContent } from "./SelectContent.vue"
export { default as SelectGroup } from "./SelectGroup.vue"
export { default as SelectItem } from "./SelectItem.vue"
export { default as SelectItemText } from "./SelectItemText.vue"
export { default as SelectLabel } from "./SelectLabel.vue"
export { default as SelectScrollDownButton } from "./SelectScrollDownButton.vue"
export { default as SelectScrollUpButton } from "./SelectScrollUpButton.vue"
export { default as SelectSeparator } from "./SelectSeparator.vue"
export { default as SelectTrigger } from "./SelectTrigger.vue"
export { default as SelectValue } from "./SelectValue.vue"

export const selectTriggerVariants = cva(
  "flex w-full items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent text-sm shadow-sm ring-offset-background data-[placeholder]:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:truncate text-start",
  {
    variants: {
      variant: {
        default: "bg-background text-foreground hover:bg-hover",
        ghost: "border-transparent text-muted-foreground hover:bg-hover-strong hover:text-accent-foreground",
        muted: "bg-muted border-transparent text-muted-foreground hover:bg-muted/80",
        primary: "bg-btn-primary text-btn-primary-foreground hover:bg-btn-primary-hover",
      },
      size: {
        default: "h-9 px-3 py-2",
        sm: "h-7 px-2 text-xs",
        xs: "h-6 w-24 gap-1 px-2.5 text-xs",
        lg: "h-10 px-4",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
)

export type SelectTriggerVariants = VariantProps<typeof selectTriggerVariants>
