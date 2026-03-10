# AI Flow Visualizer

The visualizer is a **product-facing explanation layer** for KATARA.
It makes optimization visible for every request in a live dark dashboard.

## Displayed stages

| Stage | What is shown |
| --- | --- |
| Raw context | Incoming token count and source type |
| Compiled context | Reduced token count after compiler |
| Memory reuse | Tokens saved via stable-block reuse |
| Final route | Selected provider and routing reason |
| Policy effect | Which policy influenced the decision |

## Dashboard location

The visualizer is rendered by the `FlowVisualizer.vue` component
and is accessible at `/flow` in the Vue dashboard.

## Roadmap

In V7.4, the visualizer will animate real request paths and show
per-stage token avoidance in real time.
