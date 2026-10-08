"""程序化生成 bevy_study 需要的全部素材（PNG / glTF）。

为什么用脚本生成而不是放二进制进仓库：
  · 可复现 —— 任何人跑一遍就得到同样的素材，不需要下载
  · 可审查 —— 素材长什么样，读代码就知道
  · 体积小 —— 全是简单几何图形

用法：
    python tools/make_assets.py

生成（相对仓库根目录）：
    assets/textures/logo.png      128x128   020 讲加载的单张图片
    assets/textures/runner.png    288x48    021 讲的 6 帧雪碧图（每帧 48x48）
    assets/models/pyramid.gltf    —         028 讲加载的 glTF 模型（三节点树）

字体不生成：Bevy 内置了一份默认字体，够用，避免分发字体文件。
音频不生成：见 main() 里的说明。
"""

from __future__ import annotations

import base64
import json
import math
import struct
import wave
from pathlib import Path

from PIL import Image, ImageDraw

# 仓库根目录 = 本文件所在目录的上一级
ROOT = Path(__file__).resolve().parent.parent
TEXTURES = ROOT / "assets" / "textures"
MODELS = ROOT / "assets" / "models"

SAMPLE_RATE = 44_100


# ─────────────────────────────────────────────────────────────────────
# 图片
# ─────────────────────────────────────────────────────────────────────


def make_logo(size: int = 128) -> Image.Image:
    """020 讲用的单张图片：一个带渐变的圆角方块 + 白色圆环。

    刻意做得和"找不到图时显示的纯色方块"完全不同 —— 这样一眼就能看出
    资源到底加载成功了没有。
    """
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 圆角方块 + 从上到下的青→蓝渐变
    inset = size // 12
    for y in range(inset, size - inset):
        t = (y - inset) / max(1, (size - 2 * inset))
        color = (
            int(60 + (40 - 60) * t),  # R
            int(200 + (110 - 200) * t),  # G
            int(220 + (230 - 220) * t),  # B
            255,
        )
        draw.line([(inset, y), (size - inset, y)], fill=color)
    # 把四个角削圆（用与背景同色的圆盖掉）
    radius = size // 5
    for corner in (
        (inset, inset),
        (size - inset, inset),
        (inset, size - inset),
        (size - inset, size - inset),
    ):
        draw.ellipse(
            [
                corner[0] - radius // 2,
                corner[1] - radius // 2,
                corner[0] + radius // 2,
                corner[1] + radius // 2,
            ],
            fill=(0, 0, 0, 0),
        )

    # 白色圆环
    ring = size // 3
    cy = cx = size // 2
    width = max(4, size // 16)
    draw.ellipse(
        [cx - ring, cy - ring, cx + ring, cy + ring],
        outline=(255, 255, 255, 255),
        width=width,
    )
    return img


def make_runner_sheet(frame_w: int = 48, frames: int = 6) -> Image.Image:
    """021 讲用的雪碧图：一个 6 帧的"跑步小人"。

    每帧画头、身体、两条腿，腿的角度按正弦摆动 —— 放起来就是走路循环。
    单帧 48x48，横向排成 288x48 的一整条。
    """
    sheet = Image.new("RGBA", (frame_w * frames, frame_w), (0, 0, 0, 0))
    draw = ImageDraw.Draw(sheet)

    skin = (250, 220, 170, 255)
    body_color = (235, 90, 90, 255)
    leg_color = (70, 100, 200, 255)

    for i in range(frames):
        ox = i * frame_w  # 这一帧在雪碧图里的横向偏移
        cx = frame_w // 2

        # 腿的角度：一个完整正弦周期，左右腿反相
        phase = 2 * math.pi * i / frames
        swing = math.sin(phase) * 0.5  # 弧度，约 ±28°
        hip_y = frame_w * 0.62

        for sign in (-1, 1):
            angle = swing * sign
            length = frame_w * 0.26
            x1 = cx + math.sin(angle) * length * 0.5
            y1 = hip_y
            x2 = cx + math.sin(angle) * length
            y2 = hip_y + math.cos(angle) * length
            draw.line([(x1 + ox, y1), (x2 + ox, y2)], fill=leg_color, width=5)

        # 身体
        draw.rounded_rectangle(
            [cx - 9 + ox, frame_w * 0.36, cx + 9 + ox, hip_y + 2],
            radius=6,
            fill=body_color,
        )
        # 头
        head_r = frame_w * 0.14
        draw.ellipse(
            [
                cx - head_r + ox,
                frame_w * 0.14 - head_r,
                cx + head_r + ox,
                frame_w * 0.14 + head_r,
            ],
            fill=skin,
        )

    return sheet


# ─────────────────────────────────────────────────────────────────────
# 音频
# ─────────────────────────────────────────────────────────────────────


def write_wav(path: Path, samples: list[float], sample_rate: int = SAMPLE_RATE) -> None:
    """把 -1.0~1.0 的浮点样本写成 16 位单声道 WAV。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(sample_rate)
        frames = bytearray()
        for s in samples:
            clipped = max(-1.0, min(1.0, s))
            frames += struct.pack("<h", int(clipped * 32767))
        f.writeframes(bytes(frames))


def make_beep(duration: float = 0.18, freq: float = 880.0) -> list[float]:
    """024 讲的"开火"音效：一个快速衰减的正弦音。"""
    n = int(SAMPLE_RATE * duration)
    out = []
    for i in range(n):
        t = i / SAMPLE_RATE
        # 指数衰减包络，避免结尾"啪"的爆音
        envelope = math.exp(-18.0 * t)
        out.append(0.6 * envelope * math.sin(2 * math.pi * freq * t))
    return out


def make_bgm(duration: float = 4.0) -> list[float]:
    """024 讲的循环背景乐：一段缓慢的琶音。

    刻意让**首尾振幅都接近 0**，这样循环播放时接缝听不出来。
    """
    # C 大调琶音，每个音 0.5 秒
    notes = [261.63, 329.63, 392.00, 523.25, 392.00, 329.63, 293.66, 261.63]
    note_len = duration / len(notes)
    n = int(SAMPLE_RATE * duration)
    out = []
    for i in range(n):
        t = i / SAMPLE_RATE
        idx = min(int(t / note_len), len(notes) - 1)
        freq = notes[idx]
        # 音符内的小包络（起音快、收尾慢），且整体首尾淡入淡出
        local = (t - idx * note_len) / note_len
        note_env = math.sin(math.pi * local) ** 0.6
        fade = min(1.0, t / 0.08, (duration - t) / 0.08)
        # 基频 + 一点二次谐波，听起来不那么单薄
        value = math.sin(2 * math.pi * freq * t) + 0.25 * math.sin(
            4 * math.pi * freq * t
        )
        out.append(0.22 * note_env * max(0.0, fade) * value)
    return out


# ─────────────────────────────────────────────────────────────────────
# glTF 模型（028 讲）
# ─────────────────────────────────────────────────────────────────────


def _pyramid_triangles() -> tuple[list[tuple[float, float, float]], list[list[int]]]:
    """一个四棱锥的顶点与三角形面。

    为了简单，**不用索引、每个三角形独占 3 个顶点**（所谓"平面着色"）——
    这样每个面只有一条法线，棱角分明，也省得算顶点法线的平均。
    体积代价是 18 个顶点而不是 5 个，对示例来说无所谓。
    """
    apex = (0.0, 1.0, 0.0)
    base = [
        (-0.6, 0.0, -0.6),
        (0.6, 0.0, -0.6),
        (0.6, 0.0, 0.6),
        (-0.6, 0.0, 0.6),
    ]

    # 四个侧面 + 底面（底面拆成两个三角形）
    faces: list[list[tuple[float, float, float]]] = []
    for i in range(4):
        faces.append([apex, base[i], base[(i + 1) % 4]])
    faces.append([base[0], base[1], base[2]])
    faces.append([base[0], base[2], base[3]])

    positions: list[tuple[float, float, float]] = []
    for tri in faces:
        # 用叉积算法线；再按"应有的朝外方向"校正绕序，
        # 否则渲染出来背面会被剔除（看不见）。
        p0, p1, p2 = tri
        normal = _cross(_sub(p1, p0), _sub(p2, p0))
        # 朝外方向：底面朝 -y；侧面朝远离中轴的水平方向
        if p0[1] == 0.0 and p1[1] == 0.0 and p2[1] == 0.0:
            outward = (0.0, -1.0, 0.0)
        else:
            cx = (p0[0] + p1[0] + p2[0]) / 3.0
            cz = (p0[2] + p1[2] + p2[2]) / 3.0
            outward = (cx, 0.0, cz)
        if _dot(normal, outward) < 0:
            tri = [p0, p2, p1]
        positions.extend(tri)

    triangles = [[i, i + 1, i + 2] for i in range(0, len(positions), 3)]
    return positions, triangles


def _sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def _cross(a, b):
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def _dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def _normalize(v):
    length = math.sqrt(_dot(v, v)) or 1.0
    return (v[0] / length, v[1] / length, v[2] / length)


def make_pyramid_gltf() -> str:
    """手写一份 glTF 2.0 文本（`.gltf` + 内嵌 base64 缓冲）。

    刻意不依赖任何 3D 工具：glTF 就是个 JSON，二进制数据用 base64 内嵌。
    结构是**一棵三节点的树**，用来演示"场景不是单个实体"：

        Root
        ├── PyramidA   (mesh 0, 平移 -1.2)
        └── PyramidB   (引用同一个 mesh，缩小到 0.6)

    两个节点共用一个 mesh —— 这正是 glTF 里"复用"的体现。
    """
    positions, triangles = _pyramid_triangles()
    normals = [None] * len(positions)
    for tri in triangles:
        p0, p1, p2 = (positions[i] for i in tri)
        n = _normalize(_cross(_sub(p1, p0), _sub(p2, p0)))
        for i in tri:
            normals[i] = n

    pos_bytes = b"".join(struct.pack("<3f", *p) for p in positions)
    nrm_bytes = b"".join(struct.pack("<3f", *n) for n in normals)
    buffer = pos_bytes + nrm_bytes

    count = len(positions)
    xs = [p[0] for p in positions]
    ys = [p[1] for p in positions]
    zs = [p[2] for p in positions]

    gltf = {
        "asset": {"version": "2.0", "generator": "bevy_study/tools/make_assets.py"},
        "scene": 0,
        "scenes": [{"name": "PyramidPair", "nodes": [0]}],
        "nodes": [
            {"name": "Root", "children": [1, 2]},
            {"name": "PyramidA", "mesh": 0, "translation": [-1.2, 0.0, 0.0]},
            {
                "name": "PyramidB",
                "mesh": 0,
                "translation": [1.2, 0.0, 0.0],
                "scale": [0.6, 0.6, 0.6],
            },
        ],
        "meshes": [
            {
                "name": "Pyramid",
                "primitives": [
                    {"attributes": {"POSITION": 0, "NORMAL": 1}, "material": 0, "mode": 4}
                ],
            }
        ],
        "materials": [
            {
                "name": "PyramidMaterial",
                "pbrMetallicRoughness": {
                    # 红铜色，和 027 里代码生成的几何体一眼能区分开
                    "baseColorFactor": [0.85, 0.35, 0.22, 1.0],
                    "metallicFactor": 0.3,
                    "roughnessFactor": 0.5,
                },
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,  # FLOAT
                "count": count,
                "type": "VEC3",
                # POSITION 必须给 min/max，规范强制要求
                "min": [min(xs), min(ys), min(zs)],
                "max": [max(xs), max(ys), max(zs)],
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": count,
                "type": "VEC3",
            },
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": len(pos_bytes),
                "target": 34962,  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": len(pos_bytes),
                "byteLength": len(nrm_bytes),
                "target": 34962,
            },
        ],
        "buffers": [
            {
                "byteLength": len(buffer),
                "uri": "data:application/octet-stream;base64,"
                + base64.b64encode(buffer).decode("ascii"),
            }
        ],
    }
    return json.dumps(gltf, indent=2, ensure_ascii=False)


# ─────────────────────────────────────────────────────────────────────


def main() -> None:
    TEXTURES.mkdir(parents=True, exist_ok=True)
    MODELS.mkdir(parents=True, exist_ok=True)
    # 不生成音频：bevy 默认的 audio feature 只带 vorbis（OGG），不支持 WAV，wav feature 又依赖 hound。024 讲改用程序合成的 Pitch，不需要音频文件。下面两个波形函数保留作参考。

    targets = []

    logo_path = TEXTURES / "logo.png"
    make_logo().save(logo_path)
    targets.append(logo_path)

    runner_path = TEXTURES / "runner.png"
    make_runner_sheet().save(runner_path)
    targets.append(runner_path)

    gltf_path = MODELS / "pyramid.gltf"
    gltf_path.write_text(make_pyramid_gltf(), encoding="utf-8")
    targets.append(gltf_path)

    print("已生成：")
    for path in targets:
        rel = path.relative_to(ROOT)
        print(f"  {rel}   {path.stat().st_size / 1024:.1f} KB")


if __name__ == "__main__":
    main()
