use std::slice::Iter;
use wgpu::PrimitiveTopology;

pub const MODE_COUNT: usize = 2 * 5 * 3;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum DrawMode {
    Normal(Topology),
    Shaded(Topology),
}

impl DrawMode {
    pub fn inner(self) -> Topology {
        match self {
            DrawMode::Normal(inner) => inner,
            DrawMode::Shaded(inner) => inner,
        }
    }

    pub fn inner_mut(&mut self) -> &mut Topology {
        match self {
            DrawMode::Normal(ref mut inner) => inner,
            DrawMode::Shaded(ref mut inner) => inner,
        }
    }

    pub fn normal_modes() -> Vec<DrawMode> {
        Topology::iterator()
            .copied()
            .map(|inner| DrawMode::Normal(inner))
            .collect()
    }

    pub fn shaded_modes() -> Vec<DrawMode> {
        Topology::iterator()
            .copied()
            .map(|inner| DrawMode::Shaded(inner))
            .collect()
    }
}

// this is fine
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Topology {
    PointList(PolygonMode),
    LineList(PolygonMode),
    LineStrip(PolygonMode),
    TriangleList(PolygonMode),
    TriangleStrip(PolygonMode),
    //TriangleFan(PolygonMode),
    //LineListWithAdjacency(PolygonMode),
    //LineStripWithAdjacency(PolygonMode),
    //TriangleListWithAdjacency(PolygonMode),
    //TriangleStripWithAdjacency(PolygonMode),
    //PatchList(PolygonMode),
}

impl Topology {
    pub fn inner(self) -> PolygonMode {
        match self {
            Topology::PointList(PolygonMode::Fill)
            | Topology::LineList(PolygonMode::Fill)
            | Topology::LineStrip(PolygonMode::Fill)
            | Topology::TriangleList(PolygonMode::Fill)
            | Topology::TriangleStrip(PolygonMode::Fill) => PolygonMode::Fill,
            //
            Topology::PointList(PolygonMode::Line)
            | Topology::LineList(PolygonMode::Line)
            | Topology::LineStrip(PolygonMode::Line)
            | Topology::TriangleList(PolygonMode::Line)
            | Topology::TriangleStrip(PolygonMode::Line) => PolygonMode::Line,
            //
            Topology::PointList(PolygonMode::Point)
            | Topology::LineList(PolygonMode::Point)
            | Topology::LineStrip(PolygonMode::Point)
            | Topology::TriangleList(PolygonMode::Point)
            | Topology::TriangleStrip(PolygonMode::Point) => PolygonMode::Point,
        }
    }

    pub fn set_inner(&mut self, mode: PolygonMode) {
        match self {
            Topology::PointList(ref mut inner)
            | Topology::LineList(ref mut inner)
            | Topology::LineStrip(ref mut inner)
            | Topology::TriangleList(ref mut inner)
            | Topology::TriangleStrip(ref mut inner) => *inner = mode,
        }
    }

    pub fn iterator() -> Iter<'static, Topology> {
        // generate all variants
        static TOPOLOGIES: [Topology; 15] = [
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
        ];

        TOPOLOGIES.iter()
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
        }
    }
}

impl From<Topology> for PrimitiveTopology {
    fn from(top: Topology) -> PrimitiveTopology {
        match top {
            Topology::PointList(_) => PrimitiveTopology::PointList,
            Topology::LineList(_) => PrimitiveTopology::LineList,
            Topology::LineStrip(_) => PrimitiveTopology::LineStrip,
            Topology::TriangleList(_) => PrimitiveTopology::TriangleList,
            Topology::TriangleStrip(_) => PrimitiveTopology::TriangleStrip,
        }
    }
}

impl From<DrawMode> for usize {
    fn from(mode: DrawMode) -> Self {
        let shifted: usize = match mode {
            DrawMode::Normal(_) => 0b0,
            DrawMode::Shaded(_) => 0b1111,
        };
        shifted + usize::from(mode.inner())
    }
}

impl From<Topology> for usize {
    fn from(top: Topology) -> Self {
        let shifted: usize = match top {
            Topology::PointList(_) => 0b0000,
            Topology::LineList(_) => 0b0011,
            Topology::LineStrip(_) => 0b0110,
            Topology::TriangleList(_) => 0b1001,
            Topology::TriangleStrip(_) => 0b1100,
        };
        shifted + usize::from(top.inner())
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

impl From<PolygonMode> for wgpu::PolygonMode {
    fn from(mode: PolygonMode) -> Self {
        match mode {
            PolygonMode::Fill => Self::Fill,
            PolygonMode::Line => Self::Line,
            PolygonMode::Point => Self::Point,
        }
    }
}

impl From<PolygonMode> for usize {
    fn from(mode: PolygonMode) -> Self {
        match mode {
            PolygonMode::Fill => 0b00,
            PolygonMode::Line => 0b01,
            PolygonMode::Point => 0b10,
        }
    }
}
