use std::slice::Iter;
use vulkano::pipeline::input_assembly::PrimitiveTopology;

// @HACK @FIXME
// this is a hack so I can use Topology(PolygonMode) as keys for pipeline hashmap
// might want to be more clever eventually
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Topology {
    PointList(PolygonMode),
    LineList(PolygonMode),
    LineStrip(PolygonMode),
    TriangleList(PolygonMode),
    TriangleStrip(PolygonMode),
    TriangleFan(PolygonMode),
    //LineListWithAdjacency(PolygonMode),
    //LineStripWithAdjacency(PolygonMode),
    //TriangleListWithAdjacency(PolygonMode),
    //TriangleStripWithAdjacency(PolygonMode),
    //PatchList(PolygonMode),
}

impl Topology {
    pub fn inner(&self) -> PolygonMode {
        match self {
            Topology::PointList(PolygonMode::Fill)
            | Topology::LineList(PolygonMode::Fill)
            | Topology::LineStrip(PolygonMode::Fill)
            | Topology::TriangleList(PolygonMode::Fill)
            | Topology::TriangleStrip(PolygonMode::Fill)
            | Topology::TriangleFan(PolygonMode::Fill) => PolygonMode::Fill,
            //
            Topology::PointList(PolygonMode::Line)
            | Topology::LineList(PolygonMode::Line)
            | Topology::LineStrip(PolygonMode::Line)
            | Topology::TriangleList(PolygonMode::Line)
            | Topology::TriangleStrip(PolygonMode::Line)
            | Topology::TriangleFan(PolygonMode::Line) => PolygonMode::Line,
            //
            Topology::PointList(PolygonMode::Point)
            | Topology::LineList(PolygonMode::Point)
            | Topology::LineStrip(PolygonMode::Point)
            | Topology::TriangleList(PolygonMode::Point)
            | Topology::TriangleStrip(PolygonMode::Point)
            | Topology::TriangleFan(PolygonMode::Point) => PolygonMode::Point,
        }
    }

    pub fn iterator() -> Iter<'static, Topology> {
        // generate all variants
        static TOPOLOGIES: [Topology; 18] = [
            Topology::PointList(PolygonMode::Fill),
            Topology::PointList(PolygonMode::Line),
            Topology::PointList(PolygonMode::Point),
            //
            Topology::LineList(PolygonMode::Fill),
            Topology::LineList(PolygonMode::Line),
            Topology::LineList(PolygonMode::Point),
            //
            Topology::LineStrip(PolygonMode::Fill),
            Topology::LineStrip(PolygonMode::Line),
            Topology::LineStrip(PolygonMode::Point),
            //
            Topology::TriangleList(PolygonMode::Fill),
            Topology::TriangleList(PolygonMode::Line),
            Topology::TriangleList(PolygonMode::Point),
            //
            Topology::TriangleStrip(PolygonMode::Fill),
            Topology::TriangleStrip(PolygonMode::Line),
            Topology::TriangleStrip(PolygonMode::Point),
            //
            Topology::TriangleFan(PolygonMode::Fill),
            Topology::TriangleFan(PolygonMode::Line),
            Topology::TriangleFan(PolygonMode::Point),
        ];

        TOPOLOGIES.into_iter()
    }
}

impl From<&Topology> for PrimitiveTopology {
    fn from(top: &Topology) -> PrimitiveTopology {
        match top {
            Topology::PointList(_) => PrimitiveTopology::PointList,
            Topology::LineList(_) => PrimitiveTopology::LineList,
            Topology::LineStrip(_) => PrimitiveTopology::LineStrip,
            Topology::TriangleList(_) => PrimitiveTopology::TriangleList,
            Topology::TriangleStrip(_) => PrimitiveTopology::TriangleStrip,
            Topology::TriangleFan(_) => PrimitiveTopology::TriangleFan,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}
