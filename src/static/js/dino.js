(function () {
  const canvas = document.getElementById("dino-canvas");
  const scoreEl = document.getElementById("current-score");
  const rankBody = document.getElementById("rank-body");
  const playerNameInput = document.getElementById("player-name");
  const restartBtn = document.getElementById("restart-btn");
  const styleSelect = document.getElementById("dino-style");
  const outfitSelect = document.getElementById("dino-outfit");
  if (!canvas || !scoreEl || !rankBody || !playerNameInput || !restartBtn || !styleSelect || !outfitSelect) return;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const world = {
    gravity: 0.62,
    speed: 5.2,
    groundY: canvas.height - 44,
  };

  const dino = {
    x: 72,
    y: 0,
    w: 44,
    h: 44,
    vy: 0,
    onGround: true,
  };
  dino.y = world.groundY - dino.h;

  let obstacles = [];
  let frames = 0;
  let score = 0;
  let ended = false;
  let pendingSubmit = false;
  let currentStyle = styleSelect.value;
  let currentOutfit = outfitSelect.value;

  function renderRank(list) {
    rankBody.innerHTML = "";
    if (list.length === 0) {
      const tr = document.createElement("tr");
      tr.innerHTML = "<td colspan='3'>暂无成绩，先玩一局吧。</td>";
      rankBody.appendChild(tr);
      return;
    }
    list.forEach((row, idx) => {
      const tr = document.createElement("tr");
      tr.innerHTML = `<td>#${idx + 1}</td><td>${row.nickname}</td><td>${row.score}</td>`;
      rankBody.appendChild(tr);
    });
  }

  async function loadRank() {
    try {
      const res = await fetch("/api/dino/scores?limit=10");
      if (!res.ok) throw new Error("failed");
      const data = await res.json();
      renderRank(Array.isArray(data.scores) ? data.scores : []);
    } catch (_) {
      rankBody.innerHTML = "";
      const tr = document.createElement("tr");
      tr.innerHTML = "<td colspan='3'>排行榜加载失败</td>";
      rankBody.appendChild(tr);
    }
  }

  async function submitScore() {
    if (pendingSubmit) return;
    const finalScore = Math.floor(score);
    if (finalScore <= 0) return;
    const nickname = (playerNameInput.value || "").trim() || "游客";
    pendingSubmit = true;
    try {
      const res = await fetch("/api/dino/scores", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ nickname, score: finalScore }),
      });
      if (!res.ok) throw new Error("failed");
      await loadRank();
    } catch (_) {
      // Ignore transient save errors; user can retry next round.
    } finally {
      pendingSubmit = false;
    }
  }

  function resetGame() {
    obstacles = [];
    frames = 0;
    score = 0;
    ended = false;
    dino.y = world.groundY - dino.h;
    dino.vy = 0;
    dino.onGround = true;
    scoreEl.textContent = "0";
  }

  function jump() {
    if (ended) {
      resetGame();
      return;
    }
    if (dino.onGround) {
      dino.vy = -11.2;
      dino.onGround = false;
    }
  }

  function spawnObstacle() {
    const h = 24 + Math.random() * 36;
    const w = 14 + Math.random() * 18;
    obstacles.push({
      x: canvas.width + 20,
      y: world.groundY - h,
      w,
      h,
      passed: false,
    });
  }

  function collide(a, b) {
    return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;
  }

  function drawBackground() {
    const grad = ctx.createLinearGradient(0, 0, 0, canvas.height);
    grad.addColorStop(0, "#f8fbff");
    grad.addColorStop(1, "#e7eefb");
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "#d7e2f4";
    for (let i = 0; i < 3; i += 1) {
      const x = ((frames * (0.5 + i * 0.15)) % (canvas.width + 220)) - 220;
      ctx.fillRect(canvas.width - x, 38 + i * 26, 120, 16);
    }

    ctx.strokeStyle = "#8fa7cb";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(0, world.groundY + 0.5);
    ctx.lineTo(canvas.width, world.groundY + 0.5);
    ctx.stroke();
  }

  function roundedRectPath(x, y, w, h, r) {
    const rr = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + rr, y);
    ctx.lineTo(x + w - rr, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + rr);
    ctx.lineTo(x + w, y + h - rr);
    ctx.quadraticCurveTo(x + w, y + h, x + w - rr, y + h);
    ctx.lineTo(x + rr, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - rr);
    ctx.lineTo(x, y + rr);
    ctx.quadraticCurveTo(x, y, x + rr, y);
    ctx.closePath();
  }

  function drawDino() {
    const skins = {
      classic: {
        body: "#2f7b52",
        belly: "#d6f2de",
        eye: "#ffffff",
        pupil: "#101418",
        accent: "#6be0a4",
      },
      tech: {
        body: "#2b365f",
        belly: "#dce6ff",
        eye: "#ffffff",
        pupil: "#1a56c4",
        accent: "#5ad6ff",
      },
      neon: {
        body: "#3a2758",
        belly: "#eedaff",
        eye: "#ffffff",
        pupil: "#311a57",
        accent: "#46ffd1",
      },
    };
    const p = skins[currentStyle] || skins.classic;
    const bx = dino.x;
    const by = dino.y;

    // Rounded body
    ctx.fillStyle = p.body;
    roundedRectPath(bx + 2, by + 8, 30, 30, 10);
    ctx.fill();

    // Head
    roundedRectPath(bx + 18, by + 2, 24, 20, 8);
    ctx.fill();

    // Belly
    ctx.fillStyle = p.belly;
    roundedRectPath(bx + 12, by + 16, 14, 16, 6);
    ctx.fill();

    // Tail
    ctx.fillStyle = p.body;
    ctx.beginPath();
    ctx.moveTo(bx + 4, by + 24);
    ctx.lineTo(bx - 8, by + 18);
    ctx.lineTo(bx - 2, by + 30);
    ctx.closePath();
    ctx.fill();

    // Legs
    ctx.fillRect(bx + 10, by + 34, 7, 9);
    ctx.fillRect(bx + 23, by + 34, 7, 9);

    // Eye
    ctx.fillStyle = p.eye;
    ctx.beginPath();
    ctx.arc(bx + 34, by + 10, 3.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = p.pupil;
    ctx.beginPath();
    ctx.arc(bx + 35, by + 10, 1.5, 0, Math.PI * 2);
    ctx.fill();

    // Tech accent
    ctx.strokeStyle = p.accent;
    ctx.lineWidth = 1.7;
    ctx.beginPath();
    ctx.moveTo(bx + 8, by + 15);
    ctx.lineTo(bx + 15, by + 15);
    ctx.lineTo(bx + 15, by + 24);
    ctx.stroke();

    if (currentOutfit === "visor") {
      ctx.fillStyle = "rgba(73, 196, 255, 0.42)";
      roundedRectPath(bx + 28, by + 7, 10, 6, 3);
      ctx.fill();
      ctx.strokeStyle = "#4ec9ff";
      ctx.lineWidth = 1.3;
      ctx.stroke();
    } else if (currentOutfit === "cape") {
      ctx.fillStyle = currentStyle === "neon" ? "#ff73d6" : "#ff6b6b";
      ctx.beginPath();
      ctx.moveTo(bx + 5, by + 14);
      ctx.lineTo(bx - 10, by + 18);
      ctx.lineTo(bx + 3, by + 34);
      ctx.closePath();
      ctx.fill();
    } else if (currentOutfit === "jet") {
      ctx.fillStyle = "#55607a";
      roundedRectPath(bx - 2, by + 18, 8, 14, 3);
      ctx.fill();
      ctx.fillStyle = "#ffad42";
      ctx.beginPath();
      ctx.moveTo(bx + 2, by + 33);
      ctx.lineTo(bx - 2, by + 40);
      ctx.lineTo(bx + 6, by + 40);
      ctx.closePath();
      ctx.fill();
    }
  }

  function drawObstacle(item) {
    ctx.fillStyle = "#35588f";
    ctx.fillRect(item.x, item.y, item.w, item.h);
  }

  function drawGameOver() {
    ctx.fillStyle = "rgba(13, 17, 23, 0.62)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#fff";
    ctx.textAlign = "center";
    ctx.font = "700 28px system-ui";
    ctx.fillText("游戏结束", canvas.width / 2, canvas.height / 2 - 12);
    ctx.font = "400 15px system-ui";
    ctx.fillText("按空格或点击画布重新开始", canvas.width / 2, canvas.height / 2 + 18);
    ctx.textAlign = "start";
  }

  function update() {
    frames += 1;
    if (!ended) {
      dino.vy += world.gravity;
      dino.y += dino.vy;
      if (dino.y >= world.groundY - dino.h) {
        dino.y = world.groundY - dino.h;
        dino.vy = 0;
        dino.onGround = true;
      }

      if (frames % 78 === 0) spawnObstacle();
      obstacles.forEach((it) => {
        it.x -= world.speed;
        if (!it.passed && it.x + it.w < dino.x) {
          it.passed = true;
          score += 10;
          scoreEl.textContent = String(Math.floor(score));
        }
      });
      obstacles = obstacles.filter((it) => it.x + it.w > -10);

      const box = { x: dino.x + 2, y: dino.y + 2, w: dino.w - 4, h: dino.h - 4 };
      if (obstacles.some((it) => collide(box, it))) {
        ended = true;
        submitScore().catch(() => {});
      }
    }

    drawBackground();
    obstacles.forEach(drawObstacle);
    drawDino();
    if (ended) drawGameOver();
    requestAnimationFrame(update);
  }

  document.addEventListener("keydown", (e) => {
    if (e.code === "Space") {
      e.preventDefault();
      jump();
    }
  });
  canvas.addEventListener("pointerdown", jump);
  restartBtn.addEventListener("click", resetGame);
  styleSelect.addEventListener("change", () => {
    currentStyle = styleSelect.value;
  });
  outfitSelect.addEventListener("change", () => {
    currentOutfit = outfitSelect.value;
  });

  loadRank().catch(() => {});
  resetGame();
  requestAnimationFrame(update);
})();
