/**
 * 分屏拖拽调整大小 composable
 *
 * @param {Object} options
 * @param {number} options.min - 最小百分比 (默认 20)
 * @param {number} options.max - 最大百分比 (默认 80)
 * @returns {{ startResize: Function }}
 */
export function useSplitResize({ min = 20, max = 80 } = {}) {
  /**
   * 开始拖拽调整分屏大小
   * @param {MouseEvent} e - mousedown 事件
   * @param {HTMLElement} container - 父容器元素
   * @param {import('vue').Ref<number>} pctRef - 百分比 ref
   * @param {Function} [onFit] - 拖拽结束后的回调 (如重新 fit terminal)
   */
  function startResize(e, container, pctRef, onFit) {
    e.preventDefault()
    const rect = container.getBoundingClientRect()
    const startX = e.clientX
    const startPct = pctRef.value

    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'

    const onMove = (ev) => {
      const pct = startPct + ((ev.clientX - startX) / rect.width) * 100
      pctRef.value = Math.max(min, Math.min(max, pct))
    }

    const onUp = () => {
      document.removeEventListener('mousemove', onMove)
      document.removeEventListener('mouseup', onUp)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
      onFit?.()
    }

    document.addEventListener('mousemove', onMove)
    document.addEventListener('mouseup', onUp)
  }

  return { startResize }
}
