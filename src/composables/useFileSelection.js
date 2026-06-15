import { ref } from 'vue'

/**
 * Composable for file selection logic (click + drag) used by both local and remote panels.
 * @param {import('vue').Reactive<Set<string>>} selectedSet - reactive Set of selected file paths
 * @param {import('vue').Ref<Array>} files - ref to the file list array
 * @param {object} [opts]
 * @param {function} [opts.onSelect] - callback invoked with the file path after selection changes
 */
export function useFileSelection(selectedSet, files, opts = {}) {
  const lastClicked = ref(null)

  function onClick(e, file) {
    if (e.ctrlKey || e.metaKey) {
      if (selectedSet.has(file.path)) selectedSet.delete(file.path)
      else selectedSet.add(file.path)
    } else if (e.shiftKey && lastClicked.value) {
      const paths = files.value.map(f => f.path)
      const start = paths.indexOf(lastClicked.value)
      const end = paths.indexOf(file.path)
      if (start >= 0 && end >= 0) {
        const [lo, hi] = start < end ? [start, end] : [end, start]
        selectedSet.clear()
        for (let i = lo; i <= hi; i++) selectedSet.add(paths[i])
      }
    } else {
      selectedSet.clear()
      selectedSet.add(file.path)
    }
    lastClicked.value = file.path
    opts.onSelect?.(file.path)
  }

  function onDragStart(e, file) {
    const paths = selectedSet.size > 0 && selectedSet.has(file.path)
      ? [...selectedSet]
      : [file.path]
    e.dataTransfer.setData('text/plain', JSON.stringify(paths))
    e.dataTransfer.effectAllowed = 'move'
    return paths
  }

  return { onClick, onDragStart }
}
