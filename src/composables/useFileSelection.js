import { ref } from 'vue'

/**
 * Composable for file selection logic (click + drag).
 * @param {import('vue').Reactive<Set<string>>} selectedSet
 * @param {import('vue').Ref<Array>} files
 * @param {object} [opts]
 * @param {function} [opts.onSelect]
 * @param {'local'|'remote'} [opts.dragSource]
 */
export function useFileSelection(selectedSet, files, opts = {}) {
  const lastClicked = ref(null)

  function onClick(event, file) {
    if (event.ctrlKey || event.metaKey) {
      if (selectedSet.has(file.path)) selectedSet.delete(file.path)
      else selectedSet.add(file.path)
    } else if (event.shiftKey && lastClicked.value) {
      const paths = files.value.map(item => item.path)
      const start = paths.indexOf(lastClicked.value)
      const end = paths.indexOf(file.path)
      if (start >= 0 && end >= 0) {
        const [low, high] = start < end ? [start, end] : [end, start]
        selectedSet.clear()
        for (let index = low; index <= high; index++) selectedSet.add(paths[index])
      }
    } else {
      selectedSet.clear()
      selectedSet.add(file.path)
    }

    lastClicked.value = file.path
    opts.onSelect?.(file.path)
  }

  function onDragStart(event, file) {
    const paths = selectedSet.size > 0 && selectedSet.has(file.path)
      ? [...selectedSet]
      : [file.path]

    const payload = {
      source: opts.dragSource || 'local',
      paths,
    }
    event.dataTransfer.setData('application/x-xterminal-files', JSON.stringify(payload))
    event.dataTransfer.effectAllowed = opts.dragSource === 'remote' ? 'move' : 'copy'
    return paths
  }

  return { onClick, onDragStart }
}
