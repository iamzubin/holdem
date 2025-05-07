import { useState } from "react"

type ToastVariant = "default" | "destructive"

interface Toast {
  title: string
  description: string
  variant?: ToastVariant
}

export function useToast() {
  const [toast, setToast] = useState<Toast | null>(null)

  const showToast = (toast: Toast) => {
    setToast(toast)
    setTimeout(() => setToast(null), 3000)
  }

  return {
    toast,
    showToast,
  }
}

export function toast(toast: Toast) {
  console.log(`[Toast] ${toast.title}: ${toast.description}`)
} 