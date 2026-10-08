"""程序化生成 bevy_study 需要的全部素材（PNG / WAV）。

为什么用脚本生成而不是放二进制进仓库：
  · 可复现 —— 任何人跑一遍就得到同样的素材，不需要下载
  · 可审查 —— 素材长什么样，读代码就知道
  · 体积小 —— 全是简单几何图形与正弦波

用法：
    python tools/make_assets.py

生成（相对仓库根目录）：
    assets/textures/logo.png      128x128   020 讲加载的单张图片
    assets/textures/runner.png    288x48    021 讲的 6 帧雪碧图（每帧 48x48）
    
字体不生成：Bevy 内置了一份默认字体，够用，避免分发字体文件。
"""

from __future__ import annotations

import math
import struct
import wave
from pathlib import Path

from PIL import Image, ImageDraw

# 仓库根目录 = 本文件所在目录的上一级
ROOT = Path(__file__).resolve().parent.parent
TEXTURES = ROOT / "assets" / "textures"
AUDIO = ROOT / "assets" / "audio"

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


def main() -> None:
    TEXTURES.mkdir(parents=True, exist_ok=True)
    # 不生成音频：bevy 默认的 audio feature 只带 vorbis（OGG），不支持 WAV，wav feature 又依赖 hound。024 讲改用程序合成的 Pitch，不需要音频文件。下面两个波形函数保留作参考。

    targets = []

    logo_path = TEXTURES / "logo.png"
    make_logo().save(logo_path)
    targets.append(logo_path)

    runner_path = TEXTURES / "runner.png"
    make_runner_sheet().save(runner_path)
    targets.append(runner_path)


    print("已生成：")
    for path in targets:
        rel = path.relative_to(ROOT)
        print(f"  {rel}   {path.stat().st_size / 1024:.1f} KB")


if __name__ == "__main__":
    main()
