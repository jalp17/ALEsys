<template>
  <div ref="container" class="w-full h-full bg-dark-900 relative overflow-hidden">
    <svg ref="svg" class="w-full h-full"></svg>
    <div v-if="loading" class="absolute inset-0 flex items-center justify-center">
      <Loader2 class="h-8 w-8 animate-spin text-primary-400" />
    </div>
    <div v-if="!loading && nodes.length === 0" class="absolute inset-0 flex items-center justify-center text-gray-500">
      Sin datos del grafo
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import * as d3 from 'd3'
import { Loader2 } from 'lucide-vue-next'

interface GraphNode {
  id: string
  label: string
  type: string
  x?: number
  y?: number
}

interface GraphEdge {
  source: string
  target: string
  type: string
}

const props = defineProps<{
  nodes: GraphNode[]
  edges: GraphEdge[]
  loading?: boolean
}>()

const container = ref<HTMLDivElement | null>(null)
const svg = ref<SVGSVGElement | null>(null)
const loading = ref(false)

let simulation: any = null

const renderGraph = () => {
  if (!svg.value || !container.value) return
  if (props.nodes.length === 0) return

  const width = container.value.clientWidth
  const height = container.value.clientHeight

  // Clear previous
  d3.select(svg.value).selectAll('*').remove()

  const svgEl = d3.select(svg.value)
    .attr('width', width)
    .attr('height', height)

  // Create force simulation
  const nodes = props.nodes.map(n => ({ ...n }))
  const edges = props.edges.map(e => ({ ...e }))

  simulation = d3.forceSimulation(nodes as any)
    .force('link', d3.forceLink(edges).id((d: any) => d.id).distance(100))
    .force('charge', d3.forceManyBody().strength(-300))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(30))

  // Draw edges
  const link = svgEl.append('g')
    .selectAll('line')
    .data(edges)
    .enter()
    .append('line')
    .attr('stroke', '#475569')
    .attr('stroke-width', 1)
    .attr('stroke-opacity', 0.6)

  // Draw nodes
  const node = svgEl.append('g')
    .selectAll('circle')
    .data(nodes)
    .enter()
    .append('circle')
    .attr('r', 8)
    .attr('fill', (d: any) => d.type === 'document' ? '#3b82f6' : '#10b981')
    .attr('stroke', '#1e293b')
    .attr('stroke-width', 2)
    .style('cursor', 'pointer')

  // Draw labels
  const label = svgEl.append('g')
    .selectAll('text')
    .data(nodes)
    .enter()
    .append('text')
    .text((d: any) => d.label.slice(0, 20))
    .attr('font-size', 10)
    .attr('fill', '#94a3b8')
    .attr('dx', 12)
    .attr('dy', 4)

  // Update positions on tick
  simulation.on('tick', () => {
    link
      .attr('x1', (d: any) => d.source.x)
      .attr('y1', (d: any) => d.source.y)
      .attr('x2', (d: any) => d.target.x)
      .attr('y2', (d: any) => d.target.y)

    node
      .attr('cx', (d: any) => d.x)
      .attr('cy', (d: any) => d.y)

    label
      .attr('x', (d: any) => d.x)
      .attr('y', (d: any) => d.y)
  })

  // Drag behavior
  const drag = d3.drag()
    .on('start', (event: any, d: any) => {
      if (!event.active) simulation.alphaTarget(0.3).restart()
      d.fx = d.x
      d.fy = d.y
    })
    .on('drag', (event: any, d: any) => {
      d.fx = event.x
      d.fy = event.y
    })
    .on('end', (event: any, d: any) => {
      if (!event.active) simulation.alphaTarget(0)
      d.fx = null
      d.fy = null
    })

  node.call(drag as any)
}

watch(() => [props.nodes, props.edges], () => {
  renderGraph()
}, { deep: true })

onMounted(() => {
  renderGraph()
  window.addEventListener('resize', renderGraph)
})

onUnmounted(() => {
  window.removeEventListener('resize', renderGraph)
  if (simulation) simulation.stop()
})
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.text-primary-400 { color: #60a5fa; }
</style>
