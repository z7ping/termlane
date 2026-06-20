// 拖拽文件上传到终端当前目录
export type UploadCallback = (files: File[]) => void

export function setupDropUpload(element: HTMLElement | null, onUpload: UploadCallback): void {
  if (!element) return

  element.addEventListener('dragover', (e: DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy'
    }
    element.classList.add('drag-over')
  })

  element.addEventListener('dragleave', () => {
    element.classList.remove('drag-over')
  })

  element.addEventListener('drop', (e: DragEvent) => {
    e.preventDefault()
    element.classList.remove('drag-over')

    if (e.dataTransfer) {
      const files = Array.from(e.dataTransfer.files)
      if (files.length > 0) {
        onUpload(files)
      }
    }
  })
}
