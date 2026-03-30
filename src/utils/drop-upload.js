// 拖拽文件上传到终端当前目录
export function setupDropUpload(element, onUpload) {
  if (!element) return

  element.addEventListener('dragover', (e) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'copy'
    element.classList.add('drag-over')
  })

  element.addEventListener('dragleave', () => {
    element.classList.remove('drag-over')
  })

  element.addEventListener('drop', (e) => {
    e.preventDefault()
    element.classList.remove('drag-over')

    const files = Array.from(e.dataTransfer.files)
    if (files.length > 0) {
      onUpload(files)
    }
  })
}
