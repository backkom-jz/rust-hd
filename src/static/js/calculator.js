(function () {
  const display = document.getElementById("calc-display");
  const keys = document.getElementById("calc-keys");
  if (!display || !keys) return;

  let current = "0";
  let stored = null;
  let pendingOp = null;
  let fresh = true;

  function render() {
    const s = current.replace(/\.?0+$/, "");
    display.textContent = s.length ? s : "0";
  }

  function appendDigit(d) {
    if (fresh) {
      current = d === "." ? "0." : d;
      fresh = false;
    } else {
      if (d === "." && current.includes(".")) return;
      if (current === "0" && d !== ".") current = d;
      else current += d;
    }
    render();
  }

  function doOp(next) {
    const n = parseFloat(current);
    if (Number.isNaN(n)) return;
    if (stored === null) {
      stored = n;
    } else if (pendingOp) {
      const a = stored;
      let r = a;
      if (pendingOp === "+") r = a + n;
      else if (pendingOp === "-") r = a - n;
      else if (pendingOp === "×") r = a * n;
      else if (pendingOp === "÷") r = n === 0 ? NaN : a / n;
      stored = Number.isFinite(r) ? r : 0;
      current = String(stored);
      render();
    }
    pendingOp = next === "=" ? null : next;
    fresh = true;
  }

  function clearAll() {
    current = "0";
    stored = null;
    pendingOp = null;
    fresh = true;
    render();
  }

  keys.addEventListener("click", function (e) {
    const btn = e.target.closest("button[data-act]");
    if (!btn) return;
    const act = btn.dataset.act;
    if (act === "digit") appendDigit(btn.dataset.val);
    else if (act === "op") doOp(btn.dataset.val);
    else if (act === "eq") doOp("=");
    else if (act === "clear") clearAll();
    else if (act === "del") {
      if (fresh) return;
      current = current.length > 1 ? current.slice(0, -1) : "0";
      render();
    }
  });

  render();
})();
