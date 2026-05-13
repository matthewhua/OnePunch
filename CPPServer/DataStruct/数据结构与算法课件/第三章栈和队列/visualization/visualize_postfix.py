#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
visualize_postfix.py

后缀表达式（逆波兰表示法）求值过程可视化脚本。
该脚本会把每一步的栈状态与剩余输入记录为一个图（SVG）文件，方便用于教学或者演示。

用法:
  python3 visualize_postfix.py "23 45 * +"      # 支持用空格分隔的 token（多位数安全）
  python3 visualize_postfix.py "2345*+"         # 也支持没有空格的表达方式（多位数不被识别）
  python3 visualize_postfix.py                 # 会提示输入

输出:
  在当前目录下创建子目录 `visual_steps/`，生成一系列 step_###.svg（或 .dot）文件。

依赖:
  - 优先使用 Python 包 `graphviz`（包装器）来生成 SVG（同时需要系统安装了 Graphviz 的 'dot'）。
  - 如果没有安装 `graphviz`，脚本会回退到生成纯文本 DOT 文件（.dot），方便在有 Graphviz 环境时再转换。

说明:
  - 运算符支持: + - * / %
  - 除法采用 C 风格的整数除法表现（向 0 截断）。
  - 对于错误情况（如操作数不足）会在可视化中标注并停止。
"""

from __future__ import annotations

import os
import sys
from typing import List

# 尝试导入 graphviz，如果不可用则回退
try:
    from graphviz import Source  # type: ignore

    HAS_GRAPHVIZ = True
except Exception:
    HAS_GRAPHVIZ = False

OUT_DIR = "visual_steps"


def tokenize(expr: str) -> List[str]:
    """
    将表达式分解为 token 列表。
    如果表达式包含空白字符，按空白分割（这样支持多位数）。
    否则按字符逐个解析：连续数字会聚合为多位数（尝试尽可能识别多位数）。
    """
    expr = expr.strip()
    if not expr:
        return []

    # 优先：如果有空格，则按空白拆分（更安全）
    if any(ch.isspace() for ch in expr):
        return [tok for tok in expr.split() if tok]

    # 否则尝试解析连续数字为多位数
    tokens = []
    i = 0
    n = len(expr)
    while i < n:
        ch = expr[i]
        if ch.isdigit():
            j = i + 1
            while j < n and expr[j].isdigit():
                j += 1
            tokens.append(expr[i:j])
            i = j
        else:
            # 负号作为数字的一部分只在前面或在空格分隔情形才安全，不在这里特殊处理
            tokens.append(ch)
            i += 1
    return tokens


def make_dot(
    stack: List[str], remaining: List[str], step_idx: int, action_desc: str
) -> str:
    """
    根据当前栈、剩余 token、步骤编号与操作描述，生成 Graphviz DOT（字符串）。
    栈从上到下显示，顶部在上方。
    """
    # 构建栈表格行（HTML-like label）
    if stack:
        rows = []
        for v in reversed(stack):  # 将栈顶显示在表格的第一行
            # 对 HTML 特殊字符做简单转义
            safe = (
                str(v).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            )
            rows.append(
                f'<TR><TD ALIGN="CENTER" BALIGN="CENTER" WIDTH="120">{safe}</TD></TR>'
            )
        stack_table = "\n      ".join(rows)
    else:
        stack_table = '<TR><TD ALIGN="CENTER"><i>empty</i></TD></TR>'

    remaining_label = " ".join(remaining) if remaining else "<i>--</i>"
    # DOT 文本，使用 HTML 标签构造表格
    dot = f"""digraph G {{
  rankdir=TB;
  node [shape=none];
  stack [label=<
    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0" CELLPADDING="6">
      <TR><TD BGCOLOR="#ddeeff" ALIGN="CENTER"><B>Stack (top)</B></TD></TR>
      {stack_table}
    </TABLE>
  >];
  info [shape=note, label=<
    <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="4">
      <TR><TD><B>Step</B></TD><TD>{step_idx}</TD></TR>
      <TR><TD><B>Action</B></TD><TD>{action_desc}</TD></TR>
      <TR><TD><B>Remaining</B></TD><TD>{remaining_label}</TD></TR>
    </TABLE>
  >];
  stack -> info [style=invis];
}}"""
    return dot


def ensure_outdir(path: str) -> None:
    os.makedirs(path, exist_ok=True)


def write_dot_file(dot_text: str, path: str) -> None:
    with open(path, "w", encoding="utf-8") as f:
        f.write(dot_text)


def render_svg_from_dot(dot_text: str) -> bytes:
    """
    使用 graphviz.Source 将 dot 转为 svg bytes。
    可能抛出异常，如果外部 dot 命令不可用或 graphviz 包缺失。
    """
    src = Source(dot_text)
    return src.pipe(format="svg")


def c_style_int_div(a: int, b: int) -> int:
    """
    模拟 C 语言中对整数除法的行为：向 0 截断。
    Python 的 int(a / b) 对 int 的表现等同于 C 风格（对负数向 0 截断），而 // 是向 -inf floor。
    所以使用 int(a / b) 来匹配 C 的语义。
    """
    if b == 0:
        raise ZeroDivisionError("division by zero")
    return int(a / b)


def visualize_postfix(expr: str, outdir: str = OUT_DIR) -> None:
    tokens = tokenize(expr)
    ensure_outdir(outdir)

    stack: List[str] = []
    step = 0

    def save_step(desc: str, idx_for_remaining: int) -> None:
        nonlocal step
        step += 1
        dot = make_dot(stack, tokens[idx_for_remaining:], step, desc)
        svg_path = os.path.join(outdir, f"step_{step:03d}.svg")
        dot_path = os.path.join(outdir, f"step_{step:03d}.dot")
        # 优先尝试直接渲染为 SVG，如果不可用则保存为 DOT（方便以后转换）
        if HAS_GRAPHVIZ:
            try:
                svg_bytes = render_svg_from_dot(dot)
                with open(svg_path, "wb") as f:
                    f.write(svg_bytes)
                # 同时保存 DOT 以便审阅或后续渲染
                write_dot_file(dot, dot_path)
                print(f"Wrote {svg_path}")
            except Exception as e:
                write_dot_file(dot, dot_path)
                print(f"[WARNING] 无法生成 SVG（{e}），已写入 DOT: {dot_path}")
        else:
            write_dot_file(dot, dot_path)
            print(f"[INFO] graphviz 包或系统 dot 不可用，已写入 DOT: {dot_path}")

    # --- 可视化评估流程 ---
    idx = 0
    save_step("start", idx)

    while idx < len(tokens):
        tok = tokens[idx]
        # 如果是数字（支持负号前缀的简单情形），则入栈
        if isinstance(tok, str) and tok.lstrip("-").isdigit():
            # 将数字按整数入栈（保持为 int，make_dot 会把它转为字符串）
            stack.append(int(tok))
            save_step(f"push {tok}", idx + 1)
            idx += 1
            continue

        # 非数字视为操作符
        # 先检查操作数是否足够
        if len(stack) < 2:
            save_step(f"error: not enough operands for '{tok}'", idx)
            print(
                f"Error: insufficient operands for operator '{tok}' at token index {idx}"
            )
            break

        # 从栈顶弹出两个操作数（注意顺序：先弹出的为右操作数）
        b = stack.pop()
        a = stack.pop()
        save_step(f"pop {a}, pop {b} (apply {tok})", idx)

        # 计算并处理常见运算符
        try:
            if tok == "+":
                res = a + b
            elif tok == "-":
                res = a - b
            elif tok == "*":
                res = a * b
            elif tok == "/":
                # 模拟 C 风格的整数除法（向 0 截断）
                res = c_style_int_div(a, b)
            elif tok == "%":
                res = a % b
            else:
                save_step(f"unknown operator '{tok}'", idx)
                print(f"Unknown operator '{tok}' at token index {idx}")
                break
        except ZeroDivisionError:
            save_step("error: division by zero", idx)
            print("Error: division by zero")
            break

        # 将结果入栈并保存步骤
        stack.append(res)
        save_step(f"push {res}", idx + 1)
        idx += 1

    # 结束状态
    save_step("end", len(tokens))

    # 如果最终栈中有单个结果，打印以便用户直接查看
    if len(stack) == 1:
        print("Final result:", stack[-1])
    else:
        print("Final stack:", stack)


if __name__ == "__main__":
    # 简单 CLI：第一个参数作为表达式；若无参数则交互式读入
    if len(sys.argv) >= 2:
        expr_input = sys.argv[1]
    else:
        expr_input = input(
            "请输入后缀表达式（例如: '23 45 * +' 或 '23+45*-'): "
        ).strip()

    if not expr_input:
        print("No expression provided, exiting.")
        sys.exit(0)

    try:
        visualize_postfix(expr_input, OUT_DIR)
    except Exception as e:
        print(f"[ERROR] 可视化过程中发生异常: {e}")
        #
