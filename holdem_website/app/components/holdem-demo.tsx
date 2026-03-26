'use client'
import { motion, AnimatePresence, useMotionValue } from 'motion/react'
import {
  FileText,
  FileArchive,
  AppWindow,
  ImageIcon,
  Music,
  X,
  Settings,
  Download,
  Folder,
  MousePointer2,
  ChevronLeft,
} from 'lucide-react'
import { useState, useRef, useCallback, useEffect } from 'react'
import { createPortal } from 'react-dom'

type DemoFile = {
  id: string
  name: string
  type: 'doc' | 'archive' | 'app' | 'image' | 'music' | 'folder'
  size: number
  folderId: string
}
type DragSource = 'desktop' | 'shelf'

const FILES: DemoFile[] = [
  {
    id: '1',
    name: 'IMG_0316.jpg',
    type: 'image',
    size: 3_200_000,
    folderId: 'root',
  },
  {
    id: '2',
    name: 'IMG_0417.jpg',
    type: 'image',
    size: 2_800_000,
    folderId: 'root',
  },
  {
    id: 'folder_food',
    name: 'Food',
    type: 'folder',
    size: 0,
    folderId: 'root',
  },
  { id: '4', name: 'report.pdf', type: 'doc', size: 890_000, folderId: 'root' },

  {
    id: '5',
    name: 'IMG_1381_2.jpg',
    type: 'image',
    size: 4_600_000,
    folderId: 'folder_food',
  },
  {
    id: '6',
    name: 'pizza_recipe.doc',
    type: 'doc',
    size: 1_200_000,
    folderId: 'folder_food',
  },
  {
    id: '7',
    name: 'dinner.png',
    type: 'image',
    size: 2_400_000,
    folderId: 'folder_food',
  },
]

// Spread out more since we have 90vh/vw
const POS = [
  { x: 50, y: 70 },
  { x: 50, y: 200 },
  { x: 50, y: 330 },
  { x: 50, y: 460 },
  { x: 180, y: 70 },
  { x: 180, y: 200 },
  { x: 180, y: 330 },
]
const ICONS: Record<string, typeof AppWindow> = {
  app: AppWindow,
  doc: FileText,
  archive: FileArchive,
  image: ImageIcon,
  music: Music,
  folder: Folder,
}
const COLORS: Record<string, string> = {
  app: '#60a5fa',
  doc: '#f87171',
  archive: '#facc15',
  image: '#34d399',
  music: '#c084fc',
  folder: '#60a5fa',
}

function fmt(b: number) {
  return b < 1048576
    ? `${(b / 1024).toFixed(1)} KB`
    : `${(b / 1048576).toFixed(1)} MB`
}
function Ic({ t, s = 20, id }: { t: string; s?: number; id?: string }) {
  if (t === 'image' && id) {
    const dim = 100 + parseInt(id.replace(/[^0-9]/g, '') || '0') * 10
    return (
      <img
        src={`https://placekeanu.com/${dim}/${dim}`}
        style={{ width: s, height: s, borderRadius: 4, objectFit: 'cover' }}
        draggable={false}
        alt="Keanu"
      />
    )
  }
  const C = ICONS[t]
  return <C size={s} color={COLORS[t]} />
}

function StackIcons({ files }: { files: DemoFile[] }) {
  const ic = files.slice(-5).reverse()
  if (!ic.length) return null
  return (
    <div style={{ position: 'relative', width: 32, height: 32 }}>
      {ic.map((f, i) => (
        <div
          key={f.id}
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            transform: `rotate(${-i * 10}deg) translate(${-i}px,${-i}px)`,
            zIndex: ic.length - i,
            pointerEvents: 'none',
          }}
        >
          <Ic t={f.type} id={f.id} />
        </div>
      ))}
    </div>
  )
}

function ShelfWidget({
  files,
  onClose,
  hovered,
  onHandle,
  onStackDrag,
}: {
  files: DemoFile[]
  onClose: () => void
  hovered: boolean
  onHandle: (e: React.PointerEvent) => void
  onStackDrag: (e: React.PointerEvent) => void
}) {
  const bg = hovered ? 'rgba(24,24,27,0.98)' : 'rgba(9,9,11,0.95)'
  const shadow = hovered
    ? '0 0 30px rgba(59,130,246,0.3),0 20px 60px rgba(0,0,0,0.7)'
    : '0 20px 60px rgba(0,0,0,0.5)'

  return (
    <div
      style={{
        width: 100,
        display: 'flex',
        flexDirection: 'column',
        borderRadius: 12,
        border: '1px solid rgba(63,63,70,0.5)',
        background: bg,
        backdropFilter: 'blur(24px)',
        WebkitBackdropFilter: 'blur(24px)',
        boxShadow: shadow,
        transition: 'background 0.2s,box-shadow 0.3s',
        overflow: 'hidden',
        paddingBottom: 8,
      }}
    >
      <div
        onPointerDown={onHandle}
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-end',
          height: 28,
          padding: '0 8px',
          cursor: 'grab',
        }}
      >
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: '50%',
            transform: 'translate(-50%,-50%)',
          }}
        >
          <div
            style={{
              width: 32,
              height: 3,
              borderRadius: 2,
              background: '#71717a',
            }}
          />
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation()
            onClose()
          }}
          style={{
            background: 'none',
            border: 'none',
            color: '#71717a',
            padding: 2,
            borderRadius: 3,
            cursor: 'pointer',
            lineHeight: 0,
            marginLeft: 2,
          }}
        >
          <X size={12} />
        </button>
      </div>

      <div
        onPointerDown={files.length > 0 ? onStackDrag : undefined}
        style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: 64,
          cursor: files.length > 0 ? 'grab' : 'default',
        }}
      >
        {files.length > 0 ? (
          <StackIcons files={files} />
        ) : (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              color: '#71717a',
            }}
          >
            <Download size={20} />
            <span style={{ fontSize: 9, marginTop: 4 }}>Drop here</span>
          </div>
        )}
      </div>

      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <div
          style={{
            background: '#27272a',
            borderRadius: 6,
            padding: '4px 12px',
            fontSize: 10,
            fontWeight: 500,
            color: '#d4d4d8',
          }}
        >
          {files.length} file{files.length !== 1 ? 's' : ''}
        </div>
      </div>
    </div>
  )
}

export function HoldemDemo() {
  const [mounted, setMounted] = useState(false)
  const [currentFolder, setCurrentFolder] = useState('root')
  const [shelfFiles, setShelfFiles] = useState<DemoFile[]>([])
  const [explorerFiles, setExplorerFiles] = useState<DemoFile[]>([])
  const [shelfVisible, setShelfVisible] = useState(false)

  const [drag, setDrag] = useState<{
    files: DemoFile[]
    source: DragSource
    x: number
    y: number
  } | null>(null)

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [selection, setSelection] = useState<{
    startX: number
    startY: number
    endX: number
    endY: number
  } | null>(null)

  const [shelfPos, setShelfPos] = useState({ x: 0, y: 0 })
  const shelfPosRef = useRef({ x: 0, y: 0 })

  const [isReady, setIsReady] = useState(false)
  const [explorerPos, setExplorerPos] = useState({ x: 16, y: 16 })
  const explorerPosRef = useRef({ x: 800, y: 200 })

  const [hShelf, setHShelf] = useState(false)
  const [hExplorer, setHExplorer] = useState(false)
  const [cursorVis, setCursorVis] = useState(false)

  const shelfRef = useRef<HTMLDivElement>(null)
  const explorerRef = useRef<HTMLDivElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const hShelfR = useRef(false)
  const hExpR = useRef(false)
  const visRef = useRef(false)
  const shakeR = useRef({
    d: 0 as -1 | 0 | 1,
    c: 0,
    t: null as ReturnType<typeof setTimeout> | null,
  })

  const lastTapRef = useRef<Record<string, number>>({})
  const mouseX = useMotionValue(0)
  const mouseY = useMotionValue(0)

  useEffect(() => {
    setMounted(true)
  }, [])

  useEffect(() => {
    if (!mounted || !containerRef.current) return;

    let initDone = false;
    const observer = new ResizeObserver((entries) => {
      if (initDone) return;
      for (let entry of entries) {
        const cw = entry.contentRect.width;
        const ch = entry.contentRect.height;
        if (cw > 50 && ch > 50) {
          initDone = true;
          // spawn shelf near bottom-right center initially
          const sInit = { x: Math.max(16, cw - 120), y: Math.max(16, ch - 120) }
          setShelfPos(sInit)
          shelfPosRef.current = sInit;
          
          // spawn explorer at bottom right, dynamically relative to actual rendered container dimensions
          const ex = Math.max(16, cw - 340 - 16);
          const ey = Math.max(16, ch - 420 - 16);
          const eInit = { x: ex, y: ey }
          setExplorerPos(eInit)
          explorerPosRef.current = eInit;
          setIsReady(true);
          observer.disconnect()
        }
      }
    })
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, [mounted])

  useEffect(() => {
    visRef.current = shelfVisible
  }, [shelfVisible])

  const isOver = (
    ref: React.RefObject<HTMLDivElement | null>,
    x: number,
    y: number,
  ) => {
    if (!ref.current) return false
    const r = ref.current.getBoundingClientRect()
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
  }

  const startSelection = (e: React.PointerEvent) => {
    if ((e.target as HTMLElement).id !== 'demo-bg') return
    if (!containerRef.current) return

    e.preventDefault()
    const rect = containerRef.current.getBoundingClientRect()
    const startX = e.clientX - rect.left
    const startY = e.clientY - rect.top

    setSelection({ startX, startY, endX: startX, endY: startY })
    setSelectedIds(new Set())

    const onMove = (ev: PointerEvent) => {
      const currentX = ev.clientX - rect.left
      const currentY = ev.clientY - rect.top
      setSelection({ startX, startY, endX: currentX, endY: currentY })

      const minX = Math.min(startX, currentX)
      const maxX = Math.max(startX, currentX)
      const minY = Math.min(startY, currentY)
      const maxY = Math.max(startY, currentY)

      const newSelected = new Set<string>()
      const visibleFiles = FILES.filter((f) => f.folderId === currentFolder)
      visibleFiles.forEach((f, i) => {
        const p = POS[i] || { x: 50, y: 50 }
        const cx = p.x + 45
        const cy = p.y + 40
        if (cx >= minX && cx <= maxX && cy >= minY && cy <= maxY) {
          newSelected.add(f.id)
        }
      })
      setSelectedIds(newSelected)
    }

    const onUp = () => {
      setSelection(null)
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
    }

    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
  }

  const startDrag = useCallback(
    (files: DemoFile[], source: DragSource, e: React.PointerEvent) => {
      e.preventDefault()
      e.stopPropagation()
      setDrag({ files, source, x: e.clientX, y: e.clientY })
      shakeR.current = { d: 0, c: 0, t: null }

      const onMove = (ev: PointerEvent) => {
        setDrag((p) => (p ? { ...p, x: ev.clientX, y: ev.clientY } : null))
        if (source === 'desktop') {
          const mx = ev.movementX
          if (Math.abs(mx) > 3) {
            const dir: -1 | 1 = mx > 0 ? 1 : -1
            if (shakeR.current.d !== 0 && dir !== shakeR.current.d) {
              shakeR.current.c++
              if (shakeR.current.t) clearTimeout(shakeR.current.t)
              shakeR.current.t = setTimeout(() => {
                shakeR.current.c = 0
              }, 500)
              if (shakeR.current.c >= 3 && !visRef.current) {
                // Spawn near cursor
                if (containerRef.current) {
                  const rect = containerRef.current.getBoundingClientRect()
                  const x = Math.min(
                    ev.clientX - rect.left + 20,
                    rect.width - 120,
                  )
                  const y = Math.max(ev.clientY - rect.top - 20, 0)
                  setShelfPos({ x, y })
                  shelfPosRef.current = { x, y }
                }
                setShelfVisible(true)
                shakeR.current.c = 0
              }
            }
            shakeR.current.d = dir
          }
        }
        const os = isOver(shelfRef, ev.clientX, ev.clientY)
        hShelfR.current = os
        setHShelf(os)
        const oe = isOver(explorerRef, ev.clientX, ev.clientY)
        hExpR.current = oe
        setHExplorer(oe)
      }
      const onUp = () => {
        if (source === 'desktop' && hShelfR.current) {
          setShelfFiles((p) => {
            const newFiles = [...p]
            files.forEach((f) => {
              if (!newFiles.find((existing) => existing.id === f.id))
                newFiles.push(f)
            })
            return newFiles
          })
          setSelectedIds(new Set())
        } else if (source === 'shelf' && hExpR.current) {
          setShelfFiles([])
          setExplorerFiles((p) => [
            ...p,
            ...files.filter((f) => !p.find((pf) => pf.id === f.id)),
          ])
          setShelfVisible(false)
        }
        setDrag(null)
        setHShelf(false)
        setHExplorer(false)
        hShelfR.current = false
        hExpR.current = false
        window.removeEventListener('pointermove', onMove)
        window.removeEventListener('pointerup', onUp)
      }
      window.addEventListener('pointermove', onMove)
      window.addEventListener('pointerup', onUp)
    },
    [],
  )

  const startShelfDrag = useCallback((e: React.PointerEvent) => {
    e.preventDefault()
    e.stopPropagation()
    const sx = e.clientX,
      sy = e.clientY,
      ox = shelfPosRef.current.x,
      oy = shelfPosRef.current.y
    const onMove = (ev: PointerEvent) => {
      const n = { x: ox + (ev.clientX - sx), y: oy + (ev.clientY - sy) }
      shelfPosRef.current = n
      setShelfPos(n)
    }
    const onUp = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
  }, [])

  const startExplorerDrag = useCallback((e: React.PointerEvent) => {
    e.preventDefault()
    e.stopPropagation()
    const sx = e.clientX,
      sy = e.clientY,
      ox = explorerPosRef.current.x,
      oy = explorerPosRef.current.y
    const onMove = (ev: PointerEvent) => {
      const n = { x: ox + (ev.clientX - sx), y: oy + (ev.clientY - sy) }
      explorerPosRef.current = n
      setExplorerPos(n)
    }
    const onUp = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
  }, [])

  const onContainerPointerMove = (e: React.PointerEvent) => {
    if (containerRef.current) {
      const rect = containerRef.current.getBoundingClientRect()
      mouseX.set(e.clientX - rect.left)
      mouseY.set(e.clientY - rect.top)
    }
  }

  const getTooltipContent = () => {
    if (!shelfVisible) {
      if (drag && drag.source === 'desktop')
        return (
          <>
            <span style={{ color: '#60a5fa', fontWeight: 600 }}>
              Shake mouse
            </span>{' '}
            rapidly to summon Holdem
          </>
        )
      return (
        <>
          <span style={{ color: '#e4e4e7', fontWeight: 500 }}>
            Select files
          </span>{' '}
          to get started
        </>
      )
    } else {
      if (shelfFiles.length === 0) {
        if (drag && drag.source === 'desktop')
          return (
            <>
              <span style={{ color: '#34d399', fontWeight: 600 }}>Drop it</span>{' '}
              in the shelf below
            </>
          )
        return (
          <>
            <span style={{ color: '#e4e4e7', fontWeight: 500 }}>
              Draw a box
            </span>{' '}
            to select files & drag
          </>
        )
      } else {
        if (drag && drag.source === 'shelf')
          return (
            <>
              <span style={{ color: '#c084fc', fontWeight: 600 }}>
                Move mouse
              </span>{' '}
              to the Documents folder
            </>
          )
        if (currentFolder === 'root')
          return (
            <>
              <span style={{ color: '#e4e4e7', fontWeight: 500 }}>
                Double-click Food folder
              </span>{' '}
              to add more
            </>
          )
        return (
          <>
            <span style={{ color: '#facc15', fontWeight: 600 }}>
              Click the shelf
            </span>{' '}
            to drag out files
          </>
        )
      }
    }
  }

  if (!mounted)
    return (
      <div
        style={{
          height: '80vh',
          minHeight: 600,
          borderRadius: 16,
          background: '#09090b',
          border: '1px solid #27272a',
        }}
      />
    )

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        alignItems: 'center',
        width: '100%',
      }}
    >
      <style>{`
      .holdem-demo-container, .holdem-demo-container * { cursor: none !important; }
      .hdi:hover .hdi-bg { background:rgba(255,255,255,0.05)!important; }
    `}</style>
      <div
        className="holdem-demo-container"
        ref={containerRef}
        onPointerDown={startSelection}
        onPointerMove={onContainerPointerMove}
        onPointerEnter={() => setCursorVis(true)}
        onPointerLeave={() => setCursorVis(false)}
        style={{
          width: '100%',
          position: 'relative',
          height: '80vh',
          minHeight: 600,
          borderRadius: 16,
          background: '#09090b',
          border: '1px solid #27272a',
          overflow: 'hidden',
          boxShadow: '0 25px 50px -12px rgba(0,0,0,0.5)',
          userSelect: 'none',
          WebkitUserSelect: 'none',
          touchAction: 'none',
          fontFamily: 'var(--font-geist),system-ui,sans-serif',
        }}
      >
        <div
          id="demo-bg"
          style={{
            position: 'absolute',
            inset: 0,
            opacity: 0.05,
            backgroundImage: 'radial-gradient(#fff 1px,transparent 1px)',
            backgroundSize: '32px 32px',
          }}
        />

        {/* Desktop */}
        <div style={{ position: 'absolute', inset: 0, pointerEvents: 'none' }}>
          {currentFolder !== 'root' && (
            <button
              onClick={() => {
                setCurrentFolder('root')
                setSelectedIds(new Set())
              }}
              onPointerDown={(e) => e.stopPropagation()}
              style={{
                position: 'absolute',
                left: 24,
                top: 20,
                pointerEvents: 'auto',
                background: 'rgba(24,24,27,0.85)',
                color: '#d4d4d8',
                border: '1px solid #3f3f46',
                borderRadius: 8,
                padding: '6px 14px',
                fontSize: 12,
                fontWeight: 500,
                cursor: 'none',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                backdropFilter: 'blur(8px)',
                WebkitBackdropFilter: 'blur(8px)',
              }}
            >
              <ChevronLeft size={16} /> Back to Desktop
            </button>
          )}

          {FILES.filter((f) => f.folderId === currentFolder).map((f, i) => {
            const p = POS[i] || { x: 50 + i * 20, y: 70 + i * 20 }
            const dragging =
              drag?.files.some((df) => df.id === f.id) &&
              drag.source === 'desktop'
            const isSelected = selectedIds.has(f.id)
            return (
              <div
                onDoubleClick={() => {
                  if (f.type === 'folder') {
                    setCurrentFolder(f.id)
                    setSelectedIds(new Set())
                  }
                }}
                key={f.id}
                className="hdi"
                onPointerDown={(e) => {
                  const now = Date.now()
                  if (f.type === 'folder') {
                    if (now - (lastTapRef.current[f.id] || 0) < 300) {
                      setCurrentFolder(f.id)
                      setSelectedIds(new Set())
                      return
                    }
                    lastTapRef.current[f.id] = now
                  }

                  const filesToDrag = selectedIds.has(f.id)
                    ? FILES.filter(
                        (file) =>
                          selectedIds.has(file.id) &&
                          file.folderId === currentFolder,
                      )
                    : [f]
                  if (!selectedIds.has(f.id)) setSelectedIds(new Set([f.id]))
                  startDrag(filesToDrag, 'desktop', e)
                }}
                style={{
                  position: 'absolute',
                  left: p.x,
                  top: p.y,
                  width: 90,
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: 6,
                  cursor: 'none',
                  zIndex: 5,
                  opacity: dragging ? 0.35 : 1,
                  transition: 'opacity 0.15s',
                  pointerEvents: 'auto',
                }}
              >
                <div
                  className="hdi-bg"
                  style={{
                    padding: 10,
                    borderRadius: 10,
                    transition: 'background 0.15s',
                    background: isSelected
                      ? 'rgba(59,130,246,0.15)'
                      : 'transparent',
                    border: isSelected
                      ? '1px solid rgba(59,130,246,0.5)'
                      : '1px solid transparent',
                  }}
                >
                  <Ic t={f.type} s={32} id={f.id} />
                  {f.type === 'folder' && (
                    <div
                      style={{
                        position: 'absolute',
                        width: 20,
                        height: 20,
                        background: 'rgba(0,0,0,0.4)',
                        borderRadius: '50%',
                        right: 10,
                        bottom: 20,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <span style={{ fontSize: 8, color: '#93c5fd' }}>3</span>
                    </div>
                  )}
                </div>
                <span
                  style={{
                    fontSize: 11,
                    fontWeight: 500,
                    color: isSelected ? '#fff' : '#a1a1aa',
                    textAlign: 'center',
                    background: isSelected ? '#3b82f6' : 'rgba(0,0,0,0.3)',
                    borderRadius: 4,
                    padding: '2px 6px',
                    maxWidth: 86,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {f.name}
                </span>
              </div>
            )
          })}
        </div>

        {selection && (
          <div
            style={{
              position: 'absolute',
              border: '1px solid rgba(59,130,246,0.8)',
              background: 'rgba(59,130,246,0.2)',
              pointerEvents: 'none',
              zIndex: 4,
              left: Math.min(selection.startX, selection.endX),
              top: Math.min(selection.startY, selection.endY),
              width: Math.abs(selection.endX - selection.startX),
              height: Math.abs(selection.endY - selection.startY),
            }}
          />
        )}

        {/* Explorer Document Panel */}
        <div
          ref={explorerRef}
          style={{
            position: 'absolute',
            left: explorerPos.x,
            top: explorerPos.y,
            width: 340,
            maxWidth: 'calc(100% - 32px)',
            height: 420,
            maxHeight: 'calc(100% - 32px)',
            border: '1px solid rgba(63,63,70,0.4)',
            borderRadius: 12,
            display: 'flex',
            flexDirection: 'column',
            zIndex: 10,
            background: 'rgba(9,9,11,0.7)',
            backdropFilter: 'blur(16px)',
            WebkitBackdropFilter: 'blur(16px)',
            boxShadow: '0 20px 40px rgba(0,0,0,0.4)',
            overflow: 'hidden',
            opacity: isReady ? 1 : 0,
            pointerEvents: isReady ? 'auto' : 'none',
            transition: 'opacity 0.2s'
          }}
        >
          <div
            onPointerDown={startExplorerDrag}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 10,
              padding: '12px 14px',
              borderBottom: '1px solid rgba(63,63,70,0.4)',
              background: 'rgba(24,24,27,0.9)',
              cursor: 'grab',
            }}
          >
            <div style={{ display: 'flex', gap: 5 }}>
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: 5,
                  background: '#ef4444',
                }}
              />
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: 5,
                  background: '#eab308',
                }}
              />
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: 5,
                  background: '#22c55e',
                }}
              />
            </div>
            <Folder size={14} color="#60a5fa" />
            <span style={{ fontSize: 12, fontWeight: 600, color: '#d4d4d8' }}>
              Documents
            </span>
          </div>
          <div
            style={{
              flex: 1,
              padding: 12,
              overflowY: 'auto',
              background:
                hExplorer && drag?.source === 'shelf'
                  ? 'rgba(59,130,246,0.08)'
                  : 'transparent',
              transition: 'background 0.2s',
              border:
                hExplorer && drag?.source === 'shelf'
                  ? '2px dashed rgba(59,130,246,0.4)'
                  : '2px dashed transparent',
              borderRadius: 8,
              margin: 6,
            }}
          >
            {explorerFiles.length === 0 &&
            (!drag || drag.source !== 'shelf') ? (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  height: '100%',
                  color: '#52525b',
                  gap: 8,
                }}
              >
                <Folder size={28} />
                <span style={{ fontSize: 12 }}>
                  Drop files here from Holdem
                </span>
              </div>
            ) : (
              <>
                {explorerFiles.map((f) => (
                  <div
                    key={f.id}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 10,
                      padding: '8px 10px',
                      borderRadius: 6,
                      margin: '2px 0',
                    }}
                    onMouseEnter={(e) => {
                      ;(e.currentTarget as HTMLElement).style.background =
                        'rgba(63,63,70,0.3)'
                    }}
                    onMouseLeave={(e) => {
                      ;(e.currentTarget as HTMLElement).style.background =
                        'transparent'
                    }}
                  >
                    <Ic t={f.type} s={18} id={f.id} />
                    <span
                      style={{
                        fontSize: 12,
                        color: '#e4e4e7',
                        flex: 1,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {f.name}
                    </span>
                    <span style={{ fontSize: 10, color: '#71717a' }}>
                      {fmt(f.size)}
                    </span>
                  </div>
                ))}
                {drag?.source === 'shelf' && explorerFiles.length === 0 && (
                  <div
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      justifyContent: 'center',
                      height: '100%',
                      color: '#3b82f6',
                      gap: 8,
                    }}
                  >
                    <Folder size={28} />
                    <span style={{ fontSize: 12 }}>Release to drop here</span>
                  </div>
                )}
              </>
            )}
          </div>
        </div>

        <AnimatePresence>
          {shelfVisible && (
            <motion.div
              ref={shelfRef}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              transition={{ type: 'spring', damping: 20, stiffness: 300 }}
              style={{
                position: 'absolute',
                left: shelfPos.x,
                top: shelfPos.y,
                zIndex: 25,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
              }}
            >
              <ShelfWidget
                files={shelfFiles}
                onClose={() => setShelfVisible(false)}
                hovered={hShelf && drag?.source === 'desktop'}
                onHandle={startShelfDrag}
                onStackDrag={(e) => startDrag(shelfFiles, 'shelf', e)}
              />
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence>
          {cursorVis && (
            <motion.div
              style={{
                position: 'absolute',
                left: 0,
                top: 0,
                x: mouseX,
                y: mouseY,
                translateX: -6,
                translateY: -2,
                zIndex: 9999,
                pointerEvents: 'none',
                display: 'flex',
                alignItems: 'flex-start',
                flexWrap: 'nowrap',
              }}
            >
              <MousePointer2
                size={22}
                color="#ffffff"
                fill="#09090b"
                style={{
                  filter: 'drop-shadow(0 2px 4px rgba(0,0,0,0.5))',
                  flexShrink: 0,
                }}
              />
              <div
                style={{
                  marginLeft: 6,
                  marginTop: 14,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                  background: 'rgba(24,24,27,0.85)',
                  borderRadius: 16,
                  padding: '4px 12px',
                  fontSize: 10,
                  color: '#a1a1aa',
                  backdropFilter: 'blur(12px)',
                  WebkitBackdropFilter: 'blur(12px)',
                  whiteSpace: 'nowrap',
                  border: '1px solid rgba(63,63,70,0.4)',
                  boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                  transition: 'background 0.2s',
                }}
              >
                {getTooltipContent()}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {!shelfVisible && (
        <button
          onClick={() => {
            if (containerRef.current) {
              const cw = containerRef.current.clientWidth || 1000
              const ch = containerRef.current.clientHeight || 600
              setShelfPos({
                x: Math.max(16, cw - 120),
                y: Math.max(16, ch - 120),
              })
            }
            setShelfVisible(true)
          }}
          style={{
            background: 'linear-gradient(to bottom, #27272a, #18181b)',
            border: '1px solid #3f3f46',
            borderRadius: 8,
            padding: '8px 24px',
            fontSize: 14,
            fontWeight: 500,
            color: '#d4d4d8',
            cursor: 'pointer',
            boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
            transition: 'background 0.2s',
            zIndex: 10,
          }}
        >
          Show Holdem Widget
        </button>
      )}
      {drag &&
        createPortal(
          <div
            style={{
              position: 'fixed',
              left: drag.x,
              top: drag.y,
              pointerEvents: 'none',
              zIndex: 99999,
              opacity: 0.9,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: 4,
              filter: 'drop-shadow(0 8px 16px rgba(0,0,0,0.6))',
              transform: 'translate(-50%, -50%)',
            }}
          >
            {drag.files.length > 1 ? (
              <div style={{ position: 'relative', width: 40, height: 40 }}>
                {drag.files
                  .slice(-5)
                  .reverse()
                  .map((f, i) => (
                    <div
                      key={f.id}
                      style={{
                        position: 'absolute',
                        inset: 0,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        transform: `rotate(${-i * 6}deg) translate(${-i * 2}px,${-i * 2}px)`,
                        zIndex: drag.files.length - i,
                      }}
                    >
                      <Ic t={f.type} s={32} id={f.id} />
                    </div>
                  ))}
                <div
                  style={{
                    position: 'absolute',
                    top: -6,
                    right: -6,
                    background: '#ef4444',
                    color: 'white',
                    fontSize: 10,
                    fontWeight: 600,
                    width: 20,
                    height: 20,
                    borderRadius: 10,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    border: '2px solid rgba(24,24,27,1)',
                  }}
                >
                  {drag.files.length}
                </div>
              </div>
            ) : (
              <>
                <Ic t={drag.files[0].type} s={32} id={drag.files[0].id} />
                <span
                  style={{
                    fontSize: 11,
                    fontWeight: 500,
                    color: '#e4e4e7',
                    background: 'rgba(0,0,0,0.85)',
                    borderRadius: 4,
                    padding: '2px 8px',
                    whiteSpace: 'nowrap',
                    border: '1px solid rgba(63,63,70,0.5)',
                  }}
                >
                  {drag.files[0].name}
                </span>
              </>
            )}
          </div>,
          document.body,
        )}
    </div>
  )
}
