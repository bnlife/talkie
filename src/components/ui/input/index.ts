import type { VariantProps } from "class-variance-authority"
import { cva } from "class-variance-authority"

export { default as Input } from "./Input.vue"

export const inputVariants = cva(
  "flex w-full rounded-md border border-input bg-transparent py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
  {
    variants: {
      size: {
        default: "h-9 px-3",
        sm: "h-7 px-2",
        sidebar: "h-8 pl-8",
        bare: "h-auto border-none px-1 py-0 shadow-none focus-visible:ring-0",
        inline: "h-full border-0 rounded-none shadow-none focus-visible:ring-0",
        rename: "h-auto border-none px-1 py-0.5 shadow-none rounded bg-background text-foreground ring-1 ring-ring",
      },
    },
    defaultVariants: {
      size: "default",
    },
  },
)

export type InputVariants = VariantProps<typeof inputVariants>
