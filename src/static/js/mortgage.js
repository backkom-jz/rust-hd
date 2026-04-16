(function () {
  const form = document.getElementById("mortgage-form");
  const outMonthly = document.getElementById("out-monthly");
  const outInterest = document.getElementById("out-interest");
  const outTotal = document.getElementById("out-total");
  const tbody = document.querySelector("#mortgage-table tbody");
  if (!form || !outMonthly || !tbody) return;

  function fmtMoney(n) {
    return n.toLocaleString("zh-CN", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  }

  function calc() {
    const principal = parseFloat(form.principal.value) || 0;
    const rateYear = (parseFloat(form.rate.value) || 0) / 100;
    const years = parseInt(form.years.value, 10) || 0;
    const mode = form.mode.value;
    const months = years * 12;
    const r = rateYear / 12;

    tbody.innerHTML = "";
    if (principal <= 0 || months <= 0) {
      outMonthly.textContent = "—";
      outInterest.textContent = "—";
      outTotal.textContent = "—";
      return;
    }

    let totalPay = 0;
    let totalInterest = 0;
    let firstMonth = 0;

    if (mode === "equal_principal_interest") {
      if (r === 0) {
        const m = principal / months;
        firstMonth = m;
        totalPay = principal;
        totalInterest = 0;
        for (let i = 1; i <= Math.min(months, 12); i++) {
          const tr = document.createElement("tr");
          tr.innerHTML = `<td>${i}</td><td>${fmtMoney(m)}</td><td>0.00</td><td>${fmtMoney(
            principal - m * i
          )}</td>`;
          tbody.appendChild(tr);
        }
      } else {
        const pow = Math.pow(1 + r, months);
        const monthly = (principal * r * pow) / (pow - 1);
        firstMonth = monthly;
        totalPay = monthly * months;
        totalInterest = totalPay - principal;
        let balance = principal;
        for (let i = 1; i <= Math.min(months, 12); i++) {
          const interest = balance * r;
          const principalPart = monthly - interest;
          balance -= principalPart;
          const tr = document.createElement("tr");
          tr.innerHTML = `<td>${i}</td><td>${fmtMoney(monthly)}</td><td>${fmtMoney(
            interest
          )}</td><td>${fmtMoney(Math.max(0, balance))}</td>`;
          tbody.appendChild(tr);
        }
      }
    } else {
      const principalMonthly = principal / months;
      let balance = principal;
      for (let i = 1; i <= months; i++) {
        const interest = balance * r;
        const pay = principalMonthly + interest;
        if (i === 1) firstMonth = pay;
        totalPay += pay;
        totalInterest += interest;
        balance -= principalMonthly;
        if (i <= 12) {
          const tr = document.createElement("tr");
          tr.innerHTML = `<td>${i}</td><td>${fmtMoney(pay)}</td><td>${fmtMoney(
            interest
          )}</td><td>${fmtMoney(Math.max(0, balance))}</td>`;
          tbody.appendChild(tr);
        }
      }
    }

    outMonthly.textContent = fmtMoney(firstMonth) + " 元（首月）";
    outInterest.textContent = fmtMoney(totalInterest) + " 元";
    outTotal.textContent = fmtMoney(totalPay) + " 元";
  }

  form.addEventListener("input", calc);
  form.addEventListener("change", calc);
  calc();
})();
