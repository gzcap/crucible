declare module 'force-graph' {
  export interface NodeObject {
    id: string
    name?: string
    link_count?: number
    tag_count?: number
    x?: number
    y?: number
    vx?: number
    vy?: number
    fx?: number
    fy?: number
  }

  export interface LinkObject<TNode extends NodeObject = NodeObject> {
    source: string | TNode
    target: string | TNode
  }

  export interface GraphData<TNode extends NodeObject = NodeObject, TLink extends LinkObject<TNode> = LinkObject<TNode>> {
    nodes: TNode[]
    links: TLink[]
  }

  export interface ForceGraphInstance<TNode extends NodeObject = NodeObject, TLink extends LinkObject<TNode> = LinkObject<TNode>> {
    (container: HTMLElement): this
    graphData(data: GraphData<TNode, TLink>): this
    nodeId(id: string | ((node: TNode) => string)): this
    nodeLabel(label: string | ((node: TNode) => string)): this
    nodeColor(color: string | ((node: TNode) => string)): this
    nodeSize(size: number | ((node: TNode) => number)): this
    linkColor(color: string | ((link: TLink) => string)): this
    onNodeClick(callback: (node: TNode, event: MouseEvent) => void): this
    onLinkClick(callback: (link: TLink, event: MouseEvent) => void): this
    width(w: number): this
    height(h: number): this
    destroy(): void
  }

  const ForceGraph: ForceGraphInstance
  export default ForceGraph
}