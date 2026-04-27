(function () {
  "use strict";

  const display = document.getElementById("calc-display");
  const keys = document.getElementById("calc-keys");
  if (!display || !keys) return;

  // ── State ────────────────────────────────────────────
  let current = "0";       // current input (string)
  let stored = null;       // previous operand (number)
  let pendingOp = null;    // pending operator
  let fresh = true;        // waiting for new input (after op / =)
  let lastAnswer = null;   // last result for repeated = chaining
  let error = false;       // error state (division by zero, etc.)

  // ── Helpers ──────────────────────────────────────────
  function isErrorState() {
    return error;
  }

  function setError(msg) {
    display.textContent = msg || "错误";
    display.classList.add("calc-error");
    error = true;
    stored = null;
    pendingOp = null;
    fresh = true;
  }

  function clearError() {
    if (error) {
      error = false;
      display.classList.remove("calc-error");
      current = "0";
      stored = null;
      pendingOp = null;
      fresh = true;
    }
  }

  // Format a number for display — avoid scientific notation, trim trailing zeros
  function formatNumber(n) {
    if (!Number.isFinite(n)) return null;
    // For very large/small numbers, keep enough precision
    let s = String(n);
    // If it's in scientific notation, format with toFixed or toPrecision
    if (s.includes("e")) {
      const abs = Math.abs(n);
      if (abs >= 1e15 || (abs > 0 && abs < 1e-10)) {
        // Use up to 10 significant digits
        s = n.toPrecision(10);
        // Remove trailing zeros after decimal if present
        s = s.replace(/e\+?/, "e");
      }
    }
    // Remove trailing zeros after decimal point
    if (s.includes(".")) {
      s = s.replace(/\.?0+$/, "");
    }
    return s || "0";
  }

  function render() {
    if (error) return;
    let s = current;
    // Trim trailing zeros after decimal for readability, but keep "0"
    if (s.includes(".")) {
      s = s.replace(/\.?0+$/, "");
    }
    display.textContent = s || "0";
  }

  // ── Digit / Decimal input ──────────────────────────
  function appendDigit(d) {
    clearError();
    if (fresh) {
      if (d === ".") {
        current = "0.";
      } else {
        current = d;
      }
      fresh = false;
    } else {
      // Limit input length
      if (current.replace("-", "").replace(".", "").length >= 15) return;
      if (d === ".") {
        if (current.includes(".")) return;
        current += ".";
      } else {
        // Prevent leading zeros: "0" + "0" → "0"; "0" + "5" → "5"
        if (current === "0") {
          current = d;
        } else {
          current += d;
        }
      }
    }
    render();
  }

  // ── Negate (toggle sign) ────────────────────────────
  function negate() {
    if (isErrorState()) return;
    if (current === "0") return;
    current = current.startsWith("-") ? current.slice(1) : "-" + current;
    render();
  }

  // ── Percent ─────────────────────────────────────────
  function percent() {
    if (isErrorState()) return;
    const n = parseFloat(current);
    if (Number.isNaN(n)) return;
    current = String(n / 100);
    render();
  }

  // ── Operator / Equals ──────────────────────────────
  function doOp(next) {
    clearError();
    const n = parseFloat(current);
    if (Number.isNaN(n)) return;

    if (next === "=") {
      if (pendingOp && stored !== null) {
        // Chain: compute stored op current
        const r = compute(stored, n, pendingOp);
        if (r === null) { setError("不能除以零"); return; }
        lastAnswer = r;
        current = formatNumber(r) || "0";
        stored = r;       // keep for repeated =
        // pendingOp stays, so next = re-uses stored & pendingOp
      } else if (!pendingOp && lastAnswer !== null) {
        // Repeated = after a previous calculation (no pending op)
        // Re-use: lastAnswer op current (like real calculators)
        current = formatNumber(lastAnswer) || "0";
        stored = lastAnswer;
      }
      // After displaying result, set fresh & keep stored/pendingOp for chaining
      fresh = true;
      render();
      return;
    }

    // It's an operator (+, -, ×, ÷)
    if (stored === null) {
      stored = n;
    } else if (pendingOp) {
      const r = compute(stored, n, pendingOp);
      if (r === null) { setError("不能除以零"); return; }
      stored = r;
      current = formatNumber(r) || "0";
      render();
    }
    pendingOp = next;
    lastAnswer = null;
    fresh = true;
  }

  function compute(a, b, op) {
    let r;
    switch (op) {
      case "+": r = a + b; break;
      case "-": r = a - b; break;
      case "×": r = a * b; break;
      case "÷": r = b === 0 ? NaN : a / b; break;
      default:  r = a;
    }
    if (!Number.isFinite(r)) return null;
    return r;
  }

  // ── AC: All Clear ──────────────────────────────────
  function allClear() {
    display.classList.remove("calc-error");
    current = "0";
    stored = null;
    pendingOp = null;
    fresh = true;
    lastAnswer = null;
    error = false;
    render();
  }

  // ── C / CE: Clear Entry ────────────────────────────
  function clearEntry() {
    display.classList.remove("calc-error");
    if (error) { allClear(); return; }
    current = "0";
    fresh = false;
    error = false;
    render();
  }

  // ── DEL: Backspace ─────────────────────────────────
  function backspace() {
    if (isErrorState()) return;
    if (fresh) return;
    if (current.length > 1) {
      current = current.slice(0, -1);
      if (current === "" || current === "-") current = "0";
    } else {
      current = "0";
    }
    render();
  }

  // ── Keyboard highlight helper ──────────────────────
  function flashButton(selector) {
    const btn = keys.querySelector(selector);
    if (!btn) return;
    btn.classList.add("key-active");
    setTimeout(function () { btn.classList.remove("key-active"); }, 120);
  }

  // ── Event binding: Click ───────────────────────────
  keys.addEventListener("click", function (e) {
    const btn = e.target.closest("button[data-act]");
    if (!btn) return;
    const act = btn.dataset.act;

    switch (act) {
      case "digit":
        appendDigit(btn.dataset.val);
        break;
      case "op":
        doOp(btn.dataset.val);
        break;
      case "eq":
        doOp("=");
        break;
      case "clear":
        allClear();
        break;
      case "ce":
        clearEntry();
        break;
      case "del":
        backspace();
        break;
      case "neg":
        negate();
        break;
      case "pct":
        percent();
        break;
    }
  });

  // ── Event binding: Keyboard ────────────────────────
  document.addEventListener("keydown", function (e) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;

    const key = e.key;

    // Digits & decimal point
    if (/^[0-9.]$/.test(key)) {
      e.preventDefault();
      const sel = 'button[data-act="digit"][data-val="' + key + '"]';
      flashButton(sel);
      appendDigit(key);
      return;
    }

    // Operators
    const opMap = { "+": "+", "-": "-", "*": "×", "/": "÷" };
    if (opMap[key]) {
      e.preventDefault();
      const mapped = opMap[key];
      const sel = 'button[data-act="op"][data-val="' + mapped + '"]';
      flashButton(sel);
      doOp(mapped);
      return;
    }

    // Enter / = → Equals
    if (key === "Enter" || key === "=") {
      e.preventDefault();
      flashButton('button[data-act="eq"]');
      doOp("=");
      return;
    }

    // Backspace → DEL
    if (key === "Backspace") {
      e.preventDefault();
      flashButton('button[data-act="del"]');
      backspace();
      return;
    }

    // Escape → AC, Delete → CE
    if (key === "Escape") {
      e.preventDefault();
      flashButton('button[data-act="clear"]');
      allClear();
      return;
    }
    if (key === "Delete") {
      e.preventDefault();
      flashButton('button[data-act="ce"]');
      clearEntry();
      return;
    }

    // % → Percent
    if (key === "%") {
      e.preventDefault();
      flashButton('button[data-act="pct"]');
      percent();
      return;
    }
  });

  render();
})();
