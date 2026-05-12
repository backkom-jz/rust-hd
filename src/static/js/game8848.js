(function () {
  "use strict";

  const canvas = document.getElementById("g8848-canvas");
  const heightEl = document.getElementById("g8848-height");
  const timeEl = document.getElementById("g8848-time");
  const comboEl = document.getElementById("g8848-combo");
  const scoreEl = document.getElementById("g8848-score");
  const rescueEl = document.getElementById("g8848-rescue");
  const badgeEl = document.getElementById("g8848-badge");
  const rankBody = document.getElementById("g8848-rank-body");
  const playerNameInput = document.getElementById("g8848-player-name");
  const restartBtn = document.getElementById("g8848-restart-btn");
  const summitBanner = document.getElementById("g8848-summit-banner");
  const tabTop = document.getElementById("g8848-tab-top");
  const tabSummit = document.getElementById("g8848-tab-summit");
  const pauseBtn = document.getElementById("g8848-pause-btn");
  const btnLeft = document.getElementById("g8848-btn-left");
  const btnRight = document.getElementById("g8848-btn-right");
  const mobileCtrls = document.getElementById("g8848-mobile-ctrls");

  if (
    !canvas || !heightEl || !timeEl || !comboEl || !scoreEl || !rescueEl || !badgeEl || !rankBody ||
    !playerNameInput || !restartBtn || !summitBanner || !tabTop || !tabSummit
  ) return;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  // ─── 常量 ───────────────────────────────────────────
  const PLAYER_W = 22;
  const PLAYER_H = 30;
  const GRAVITY = 0.50;
  const JUMP_FORCE = -9.5;
  const MAX_FALL = 11;
  const MOVE_SPEED = 4.5;
  const GROUND_ACCEL = 0.34;
  const AIR_ACCEL = 0.18;
  const GROUND_FRICTION = 0.82;
  const AIR_DRAG = 0.96;
  const COYOTE_FRAMES = 7;
  const JUMP_BUFFER_FRAMES = 7;
  const JUMP_CUT_MULT = 0.58;
  const PLATFORM_H = 8;
  const SCROLL_THRESHOLD = canvas.height * 0.38;
  const HEIGHT_PER_SCROLL = 1;
  const COMBO_WINDOW_FRAMES = 84;
  const RESCUE_SAFE_Y = canvas.height * 0.5;
  const MAX_PLATFORMS = 30;

  const BADGES = [
    { min: 0,    title: "登山新手",   emoji: "🥾", color: "#9ca3af" },
    { min: 100,  title: "山脚漫步",   emoji: "🌿", color: "#6b7280" },
    { min: 500,  title: "半山行者",   emoji: "🏃", color: "#f59e0b" },
    { min: 1000, title: "登山达人",   emoji: "⛰️", color: "#10b981" },
    { min: 2000, title: "攀岩高手",   emoji: "🧗", color: "#3b82f6" },
    { min: 4000, title: "雪线勇士",   emoji: "❄️", color: "#8b5cf6" },
    { min: 6000, title: "高原征服者", emoji: "🏔️", color: "#ec4899" },
    { min: 8000, title: "巅峰挑战者", emoji: "⚡", color: "#f97316" },
    { min: 8848, title: "珠峰登顶者", emoji: "👑", color: "#e8b832" },
  ];

  // ─── 状态 ───────────────────────────────────────────
  const player = { x: 0, y: 0, vx: 0, vy: 0, onPlatform: false, dir: 1 };
  const keys = { left: false, right: false };

  let platforms = [];
  let height = 0;
  let gameTime = 0;
  let combo = 0;
  let score = 0;
  let frames = 0;
  let ended = false;
  let paused = false;
  let pendingSubmit = false;
  let activeTab = "top";

  let particles = [];
  let coyoteFrames = 0;
  let jumpBufferFrames = 0;
  let jumpHeld = false;
  let cameraShake = 0;
  let lastLandFrame = -9999;
  let rescueUsed = false;
  let hintText = "";
  let hintColor = "#ffffff";
  let hintUntil = 0;

  // 新增状态
  let stars = [];
  let clouds = [];
  let summitCelebration = 0;
  let badgeFlash = 0;
  let currentBadge = BADGES[0];
  let bestHeight = 0;
  let bestScore = 0;
  let highestPlatformY = 0;

  // 本地最佳记录
  try {
    const saved = JSON.parse(localStorage.getItem("g8848_best") || "{}");
    bestHeight = saved.height || 0;
    bestScore = saved.score || 0;
  } catch (_) {}

  function saveBest() {
    try {
      localStorage.setItem("g8848_best", JSON.stringify({ height: bestHeight, score: bestScore }));
    } catch (_) {}
  }

  // 检测触摸设备，显示移动端按钮
  const isTouchDevice = "ontouchstart" in window || navigator.maxTouchPoints > 0;
  if (isTouchDevice && mobileCtrls) {
    mobileCtrls.style.display = "flex";
  }

  // ─── 预生成星星 ─────────────────────────────────────
  function generateStars() {
    stars = [];
    for (let i = 0; i < 60; i++) {
      stars.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height * 0.55,
        size: 0.8 + Math.random() * 2.2,
        twinkle: Math.random() * Math.PI * 2,
        speed: 0.02 + Math.random() * 0.04,
      });
    }
  }
  generateStars();

  // ─── 预生成云层 ─────────────────────────────────────
  function generateClouds() {
    clouds = [];
    for (let i = 0; i < 8; i++) {
      clouds.push({
        x: Math.random() * canvas.width,
        y: 30 + Math.random() * canvas.height * 0.7,
        w: 60 + Math.random() * 120,
        h: 18 + Math.random() * 22,
        speed: 0.15 + Math.random() * 0.4,
        alpha: 0.04 + Math.random() * 0.08,
      });
    }
  }
  generateClouds();

  // ─── 段位 ───────────────────────────────────────────
  function getBadge(h) {
    let b = BADGES[0];
    for (let i = BADGES.length - 1; i >= 0; i--) {
      if (h >= BADGES[i].min) { b = BADGES[i]; break; }
    }
    return b;
  }

  function updateBadge(h) {
    const b = getBadge(h);
    if (b !== currentBadge) {
      badgeFlash = 45;
      currentBadge = b;
    }
    badgeEl.textContent = b.emoji + " " + b.title;
    badgeEl.style.background = b.color + "22";
    badgeEl.style.borderColor = b.color + "44";
    badgeEl.style.color = b.color;
  }

  function setHint(text, color, durationFrames) {
    hintText = text;
    hintColor = color || "#ffffff";
    hintUntil = frames + (durationFrames || 90);
  }

  function applyComboScore(baseScore) {
    const bonusMul = 1 + Math.max(0, combo - 1) * 0.2;
    const add = Math.floor(baseScore * bonusMul);
    score += add;
    scoreEl.textContent = String(score);
    if (combo >= 4) {
      setHint("连击 x" + combo + " +" + add, "#ffd166", 50);
    }
  }

  // ─── 平台生成（改进版）───────────────────────────────
  function initPlatforms() {
    platforms = [];
    platforms.push({ x: 0, y: canvas.height - 18, w: canvas.width, h: 18, type: "ground", vx: 0 });

    // 起始平台：更密集、更宽，降低前期难度
    let baseY = canvas.height - 70;
    for (let i = 0; i < 8; i++) {
      const gap = 28 + Math.random() * 18;
      baseY -= gap;
      addPlatform(baseY, true);
    }
    platforms.sort(function (a, b) { return b.y - a.y; });
    highestPlatformY = platforms[0].y;
  }

  function addPlatform(baseY, initial) {
    // 宽度随高度逐渐减小，前期保持较宽，中后期逐步收窄
    const heightFactor = Math.max(0, (height - 400) / 8000);
    const maxW = initial ? 130 : Math.max(48, 115 - heightFactor * 70);
    const minW = initial ? 65 : Math.max(35, 55 - heightFactor * 22);
    const w = minW + Math.random() * (maxW - minW);
    const x = Math.random() * (canvas.width - w);
    let type = "normal";
    let vx = 0;

    if (!initial) {
      const r = Math.random();
      if (height > 600 && r < 0.08) {
        type = "bouncy";
      } else if (height > 1200 && r < 0.14) {
        type = "moving";
        vx = (Math.random() > 0.5 ? 1 : -1) * (0.6 + Math.random() * 1.4);
      } else if (height > 3000 && r < 0.07) {
        type = "fragile";
      }
    }

    platforms.push({ x: x, y: baseY, w: w, h: PLATFORM_H, type: type, vx: vx, crackTimer: 0 });
    if (baseY < highestPlatformY) highestPlatformY = baseY;
  }

  // ─── 脆冰平台碎裂动画 ───────────────────────────────
  function crackFragilePlatform(pl) {
    if (pl.crackTimer > 0) return; // 已经在碎裂中
    pl.crackTimer = 18; // 18帧后碎裂
    setHint("冰面碎裂！快跳！", "#f87171", 40);
  }

  // ─── 排行榜 ─────────────────────────────────────────
  function escapeHtml(s) {
    const d = document.createElement("div");
    d.appendChild(document.createTextNode(s));
    return d.innerHTML;
  }

  function renderRank(list) {
    rankBody.innerHTML = "";
    if (!list || list.length === 0) {
      const tr = document.createElement("tr");
      tr.innerHTML = "<td colspan='5'>暂无成绩，挑战一局吧！</td>";
      rankBody.appendChild(tr);
      return;
    }
    list.forEach(function (row, idx) {
      const tr = document.createElement("tr");
      const ht = row.reached_8848 ? "🏔️ 8848" : Math.round(row.height) + "m";
      const badgeHtml = row.reached_8848
        ? '<span class="badge-summit">登顶</span>'
        : "-";
      const dur = (row.duration_sec || 0) + "s";
      tr.innerHTML =
        "<td>#" + (idx + 1) + "</td>" +
        "<td>" + escapeHtml(row.nickname) + "</td>" +
        "<td>" + ht + "</td>" +
        "<td>" + dur + "</td>" +
        "<td>" + badgeHtml + "</td>";
      rankBody.appendChild(tr);
    });
  }

  async function loadRank() {
    const endpoint = activeTab === "summit"
      ? "/api/game8848/scores?limit=50"
      : "/api/game8848/scores?limit=10";
    try {
      const res = await fetch(endpoint);
      if (!res.ok) throw new Error("fail");
      const data = await res.json();
      let list = Array.isArray(data.scores) ? data.scores : [];
      if (activeTab === "summit") {
        list = list.filter(function (r) { return r.reached_8848; });
      }
      renderRank(list);
    } catch (_) {
      rankBody.innerHTML = "";
      const tr = document.createElement("tr");
      tr.innerHTML = "<td colspan='5'>排行榜加载失败</td>";
      rankBody.appendChild(tr);
    }
  }

  async function submitScore(finalHeight, durationSec, reachedSummit) {
    if (pendingSubmit) return;
    const nickname = (playerNameInput.value || "").trim() || "登山者";
    const finalH = Math.round(finalHeight * 10) / 10;
    if (finalH <= 0) return;
    pendingSubmit = true;
    try {
      const res = await fetch("/api/game8848/scores", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          nickname: nickname,
          height: finalH,
          duration_sec: durationSec,
          reached_8848: reachedSummit,
        }),
      });
      if (!res.ok) throw new Error("fail");
      await loadRank();
    } catch (_) {}
    finally { pendingSubmit = false; }
  }

  // ─── 粒子系统（优化版）───────────────────────────────
  function spawnSnow() {
    if (frames % 4 !== 0) return;
    // 高海拔雪更多
    const snowChance = height > 3000 ? 1 : (height > 1000 ? 0.6 : 0.35);
    if (Math.random() > snowChance) return;
    particles.push({
      x: Math.random() * canvas.width,
      y: -5,
      vy: 0.3 + Math.random() * 0.6,
      vx: -0.3 + Math.random() * 0.6,
      r: 1 + Math.random() * 2.5,
      life: 300 + Math.random() * 200,
      alpha: 0.3 + Math.random() * 0.5,
    });
  }

  function emitBurst(x, y, count, color) {
    for (let i = 0; i < count; i++) {
      particles.push({
        x: x + (Math.random() - 0.5) * 10,
        y: y + (Math.random() - 0.5) * 6,
        vy: -1.4 + Math.random() * 1.6,
        vx: -1.8 + Math.random() * 3.6,
        g: 0.05 + Math.random() * 0.06,
        r: 1 + Math.random() * 2.8,
        life: 26 + Math.random() * 20,
        alpha: 0.45 + Math.random() * 0.45,
        color: color,
      });
    }
  }

  function emitFirework(x, y) {
    const colors = ["#f6d365", "#fda085", "#fbbf24", "#34d399", "#60a5fa", "#a78bfa", "#f472b6", "#fb923c"];
    for (let i = 0; i < 30; i++) {
      const angle = Math.random() * Math.PI * 2;
      const speed = 1.5 + Math.random() * 4;
      const col = colors[Math.floor(Math.random() * colors.length)];
      particles.push({
        x: x, y: y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed - 2,
        g: 0.08 + Math.random() * 0.04,
        r: 1.5 + Math.random() * 2.2,
        life: 50 + Math.random() * 40,
        alpha: 0.6 + Math.random() * 0.3,
        color: col,
      });
    }
  }

  function updateParticles() {
    // 反向遍历，标记删除
    for (let i = particles.length - 1; i >= 0; i--) {
      const p = particles[i];
      p.vy += p.g || 0;
      p.x += p.vx;
      p.y += p.vy;
      p.life -= 1;
      if (p.life <= 0 || p.y > canvas.height + 20 || p.x < -20 || p.x > canvas.width + 20) {
        particles.splice(i, 1);
      }
    }
    // 限制粒子总数
    if (particles.length > 200) {
      particles.splice(0, particles.length - 200);
    }
  }

  function drawParticles() {
    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      const alpha = p.alpha * Math.min(1, p.life / 100);
      if (p.color) {
        ctx.globalAlpha = alpha;
        ctx.fillStyle = p.color;
      } else {
        ctx.fillStyle = "rgba(255,255,255," + alpha + ")";
      }
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }

  // ─── 重置 ───────────────────────────────────────────
  function resetGame() {
    initPlatforms();
    player.x = canvas.width / 2 - PLAYER_W / 2;
    player.y = canvas.height - 80;
    player.vx = 0;
    player.vy = 0;
    player.onPlatform = false;
    player.dir = 1;
    height = 0;
    gameTime = 0;
    combo = 0;
    score = 0;
    frames = 0;
    ended = false;
    paused = false;
    particles = [];
    coyoteFrames = 0;
    jumpBufferFrames = 0;
    jumpHeld = false;
    cameraShake = 0;
    lastLandFrame = -9999;
    rescueUsed = false;
    hintText = "";
    hintUntil = 0;
    summitCelebration = 0;
    badgeFlash = 0;
    currentBadge = BADGES[0];
    highestPlatformY = 0;
    summitBanner.classList.remove("show");
    heightEl.textContent = "0";
    timeEl.textContent = "0";
    comboEl.textContent = "0";
    scoreEl.textContent = "0";
    rescueEl.textContent = "救援：可用";
    rescueEl.style.color = "";
    updateBadge(0);
    if (pauseBtn) pauseBtn.textContent = "⏸";
    generateStars();
    generateClouds();
  }

  function executeJump(multiplier) {
    player.vy = JUMP_FORCE * (multiplier || 1);
    player.onPlatform = false;
    coyoteFrames = 0;
    jumpBufferFrames = 0;
    emitBurst(player.x + PLAYER_W / 2, player.y + PLAYER_H, 9, "#bfe3ff");
  }

  function jump() {
    if (ended) { resetGame(); return; }
    if (paused) return;
    jumpBufferFrames = JUMP_BUFFER_FRAMES;
    jumpHeld = true;
  }

  function togglePause() {
    if (ended) return;
    paused = !paused;
    if (pauseBtn) pauseBtn.textContent = paused ? "▶" : "⏸";
  }

  // ─── 绘制 ───────────────────────────────────────────
  function drawBackground() {
    // 天空渐变 — 随高度从夜空渐变到蓝天
    const grad = ctx.createLinearGradient(0, 0, 0, canvas.height);
    const skyPct = Math.min(1, height / 3000);
    const topR = Math.round(15 + skyPct * 80);
    const topG = Math.round(23 + skyPct * 100);
    const topB = Math.round(42 + skyPct * 130);
    grad.addColorStop(0, "rgb(" + topR + "," + topG + "," + topB + ")");
    grad.addColorStop(0.35, "rgb(" + Math.round(30 + skyPct * 100) + "," + Math.round(41 + skyPct * 120) + "," + Math.round(59 + skyPct * 140) + ")");
    grad.addColorStop(0.65, "rgb(" + Math.round(51 + skyPct * 80) + "," + Math.round(62 + skyPct * 100) + "," + Math.round(85 + skyPct * 110) + ")");
    grad.addColorStop(0.85, "#5b7f9a");
    grad.addColorStop(1, "#d4e6f1");
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // 星星（使用预生成位置）
    if (height > 100) {
      const starAlpha = Math.min(0.7, 0.15 + height / 5000);
      for (let si = 0; si < stars.length; si++) {
        const s = stars[si];
        const sy = s.y - (height * 0.02) % (canvas.height * 0.55);
        const adjY = ((sy % (canvas.height * 0.55)) + canvas.height * 0.55) % (canvas.height * 0.55);
        if (adjY < 10) continue;
        const twinkle = Math.sin(frames * s.speed + s.twinkle) * 0.3 + 0.7;
        ctx.fillStyle = "rgba(255,255,255," + (starAlpha * twinkle) + ")";
        ctx.fillRect(s.x, adjY, s.size, s.size);
      }
    }

    // 云层
    for (let ci = 0; ci < clouds.length; ci++) {
      const c = clouds[ci];
      const cx = ((c.x - height * 0.01 * c.speed) % (canvas.width + 200)) - 100;
      ctx.fillStyle = "rgba(255,255,255," + c.alpha + ")";
      ctx.beginPath();
      ctx.ellipse(cx, c.y, c.w / 2, c.h / 2, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.beginPath();
      ctx.ellipse(cx - c.w * 0.25, c.y + c.h * 0.2, c.w * 0.3, c.h * 0.35, 0, 0, Math.PI * 2);
      ctx.fill();
    }

    // 远山（视差滚动）
    ctx.fillStyle = "rgba(255,255,255,0.06)";
    const mOff = (height * 0.015) % 300;
    for (let mi = 0; mi < 6; mi++) {
      const mx = mi * 180 - mOff - 40;
      const mh = 50 + Math.sin(mi * 2.1 + height * 0.001) * 25;
      ctx.beginPath();
      ctx.moveTo(mx, canvas.height - 60);
      ctx.lineTo(mx + 60, canvas.height - 60 - mh);
      ctx.lineTo(mx + 120, canvas.height - 60);
      ctx.closePath();
      ctx.fill();
    }

    // 高度显示
    const dh = Math.round(height);
    ctx.fillStyle = "rgba(255,255,255,0.25)";
    ctx.font = "bold 13px system-ui";
    ctx.textAlign = "right";
    ctx.fillText(dh + "m", canvas.width - 10, 18);
    ctx.textAlign = "start";

    // 进度条
    const prog = Math.min(1, height / 8848);
    if (prog > 0.02) {
      ctx.fillStyle = "rgba(255,255,255,0.08)";
      ctx.fillRect(10, 22, canvas.width - 20, 5);
      const progColor = prog >= 1 ? "#f6d365" : "#e8b832";
      ctx.fillStyle = progColor;
      ctx.fillRect(10, 22, (canvas.width - 20) * prog, 5);
    }
  }

  function drawPlayer() {
    const px = player.x;
    const py = player.y;
    const bounce = player.onPlatform ? 0 : Math.sin(frames * 0.25) * 1.5;
    const flip = player.dir === -1;

    ctx.save();
    if (flip) {
      ctx.translate(px + PLAYER_W / 2, 0);
      ctx.scale(-1, 1);
      ctx.translate(-(px + PLAYER_W / 2), 0);
    }

    // 身体
    ctx.fillStyle = "#3b82f6";
    roundedRectPath(px + 2, py + 8 + bounce, 18, 18, 5);
    ctx.fill();

    // 头部
    ctx.fillStyle = "#fbbf24";
    ctx.beginPath();
    ctx.arc(px + 11, py + 4 + bounce, 7, 0, Math.PI * 2);
    ctx.fill();

    // 护目镜
    ctx.fillStyle = "rgba(59, 130, 246, 0.55)";
    roundedRectPath(px + 5, py + 2 + bounce, 12, 5, 3);
    ctx.fill();

    // 手臂摆动
    const armSwing = player.onPlatform && (keys.left || keys.right) ? Math.sin(frames * 0.15) * 3 : 0;
    ctx.strokeStyle = "#fbbf24";
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.moveTo(px + 3, py + 14 + bounce + armSwing);
    ctx.lineTo(px - 2, py + 20 + bounce);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(px + 19, py + 14 + bounce - armSwing);
    ctx.lineTo(px + 24, py + 20 + bounce);
    ctx.stroke();

    // 腿摆动
    const legSwing = player.onPlatform && (keys.left || keys.right) ? Math.sin(frames * 0.15) * 4 : 0;
    ctx.strokeStyle = "#2563eb";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(px + 7, py + 26 + bounce);
    ctx.lineTo(px + 5 + legSwing, py + 34 + bounce);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(px + 15, py + 26 + bounce);
    ctx.lineTo(px + 17 - legSwing, py + 34 + bounce);
    ctx.stroke();

    // 背包
    ctx.fillStyle = "#10b981";
    roundedRectPath(px + 16, py + 12 + bounce, 8, 14, 3);
    ctx.fill();

    ctx.restore();
  }

  function roundedRectPath(x, y, w, h, r) {
    const rr = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + rr, y); ctx.lineTo(x + w - rr, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + rr);
    ctx.lineTo(x + w, y + h - rr);
    ctx.quadraticCurveTo(x + w, y + h, x + w - rr, y + h);
    ctx.lineTo(x + rr, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - rr);
    ctx.lineTo(x, y + rr);
    ctx.quadraticCurveTo(x, y, x + rr, y);
    ctx.closePath();
  }

  function drawPlatforms() {
    for (let i = 0; i < platforms.length; i++) {
      const pl = platforms[i];

      if (pl.type === "ground") {
        ctx.fillStyle = "#3d4d5f";
        roundedRectPath(pl.x, pl.y, pl.w, pl.h, 3);
        ctx.fill();
        continue;
      }

      const ice = Math.min(1, height / 5000);
      const r = Math.round(100 + (220 - 100) * ice);
      const g = Math.round(140 + (230 - 140) * ice);
      const b = Math.round(200 + (255 - 200) * ice);

      switch (pl.type) {
        case "bouncy":
          ctx.fillStyle = "#f59e0b";
          roundedRectPath(pl.x, pl.y, pl.w, pl.h, 3);
          ctx.fill();
          // 弹簧线圈
          ctx.fillStyle = "rgba(255,255,255,0.4)";
          for (let si = 0; si < 3; si++) {
            ctx.fillRect(pl.x + 4 + si * 8, pl.y + 2, 4, 4);
          }
          break;
        case "moving":
          ctx.fillStyle = "#8b5cf6";
          roundedRectPath(pl.x, pl.y, pl.w, pl.h, 3);
          ctx.fill();
          ctx.fillStyle = "rgba(255,255,255,0.25)";
          ctx.fillRect(pl.x + 4, pl.y + 2, pl.w - 8, 2);
          break;
        case "fragile":
          // 碎裂动画：根据 crackTimer 改变外观
          if (pl.crackTimer > 0) {
            const crackPct = 1 - pl.crackTimer / 18;
            const shake = Math.sin(frames * 1.2) * crackPct * 2;
            ctx.fillStyle = "rgb(" + Math.round(160 + crackPct * 60) + "," + Math.round(170 + crackPct * 50) + "," + Math.round(190 + crackPct * 30) + ")";
            roundedRectPath(pl.x + shake, pl.y, pl.w, pl.h, 3);
            ctx.fill();
            // 裂缝扩展
            ctx.strokeStyle = "rgba(255,80,80," + (0.4 + crackPct * 0.5) + ")";
            ctx.lineWidth = 0.8 + crackPct * 1.2;
            ctx.beginPath();
            ctx.moveTo(pl.x + 5, pl.y + 2);
            ctx.lineTo(pl.x + pl.w * (0.3 + crackPct * 0.3), pl.y + pl.h / 2);
            ctx.lineTo(pl.x + pl.w * 0.5, pl.y + pl.h - 1);
            ctx.lineTo(pl.x + pl.w * (0.65 + crackPct * 0.2), pl.y + 1);
            ctx.stroke();
          } else {
            ctx.fillStyle = "#94a3b8";
            roundedRectPath(pl.x, pl.y, pl.w, pl.h, 3);
            ctx.fill();
            ctx.strokeStyle = "rgba(255,255,255,0.3)";
            ctx.lineWidth = 0.8;
            ctx.beginPath();
            ctx.moveTo(pl.x + 5, pl.y + 2);
            ctx.lineTo(pl.x + pl.w / 2, pl.y + pl.h - 2);
            ctx.lineTo(pl.x + pl.w - 5, pl.y + 2);
            ctx.stroke();
          }
          break;
        default:
          ctx.fillStyle = "rgb(" + r + "," + g + "," + b + ")";
          roundedRectPath(pl.x, pl.y, pl.w, pl.h, 3);
          ctx.fill();
      }

      ctx.strokeStyle = "rgba(255,255,255,0.2)";
      ctx.lineWidth = 0.8;
      ctx.stroke();
    }
  }

  function drawGameOver() {
    // 半透明覆盖
    ctx.fillStyle = "rgba(0,0,0,0.65)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const b = getBadge(height);
    const cy = canvas.height / 2;

    // 标题
    ctx.fillStyle = "#fff";
    ctx.textAlign = "center";
    ctx.font = "700 30px system-ui";
    ctx.fillText("游戏结束", canvas.width / 2, cy - 55);

    // 段位
    ctx.font = "18px system-ui";
    ctx.fillStyle = b.color;
    ctx.fillText(b.emoji + " " + b.title, canvas.width / 2, cy - 22);

    // 统计信息
    ctx.font = "400 14px system-ui";
    ctx.fillStyle = "rgba(255,255,255,0.85)";
    const reached = Math.round(height);
    const dur = Math.floor(gameTime);
    ctx.fillText(
      "高度: " + reached + "m  |  用时: " + dur + "s  |  连击: " + combo + "x  |  积分: " + score,
      canvas.width / 2, cy + 10
    );

    // 新纪录提示
    let newRecord = false;
    if (reached > bestHeight) {
      bestHeight = reached;
      newRecord = true;
    }
    if (score > bestScore) {
      bestScore = score;
      newRecord = true;
    }
    saveBest();

    if (newRecord && bestHeight > 0) {
      ctx.fillStyle = "#f6d365";
      ctx.font = "700 15px system-ui";
      ctx.fillText("🎉 新个人纪录！", canvas.width / 2, cy + 35);
    } else if (bestHeight > 0) {
      ctx.fillStyle = "rgba(255,255,255,0.5)";
      ctx.font = "400 13px system-ui";
      ctx.fillText("个人最佳: " + bestHeight + "m | " + bestScore + "分", canvas.width / 2, cy + 35);
    }

    // 登顶提示
    if (height >= 8848) {
      ctx.fillStyle = "#f6d365";
      ctx.font = "700 16px system-ui";
      ctx.fillText("🏔️ 恭喜登顶珠穆朗玛峰！🏔️", canvas.width / 2, cy + 58);
    }

    // 操作提示
    ctx.font = "400 13px system-ui";
    ctx.fillStyle = "rgba(255,255,255,0.5)";
    ctx.fillText("点击画布或按空格重新挑战", canvas.width / 2, cy + 80);
    ctx.textAlign = "start";
  }

  function drawPauseOverlay() {
    ctx.fillStyle = "rgba(0,0,0,0.5)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#fff";
    ctx.textAlign = "center";
    ctx.font = "700 28px system-ui";
    ctx.fillText("⏸ 暂停中", canvas.width / 2, canvas.height / 2 - 5);
    ctx.font = "400 14px system-ui";
    ctx.fillStyle = "rgba(255,255,255,0.6)";
    ctx.fillText("按 P 或点击暂停按钮继续", canvas.width / 2, canvas.height / 2 + 25);
    ctx.textAlign = "start";
  }

  function drawSummit() {
    // 旋转光环
    const t = frames * 0.04;
    for (let i = 0; i < 20; i++) {
      const angle = t + (Math.PI * 2 * i) / 20;
      const radius = 55 + Math.sin(t * 2 + i * 1.5) * 30;
      const cx = canvas.width / 2 + Math.cos(angle) * radius;
      const cy = canvas.height / 3 + Math.sin(angle) * radius * 0.35;
      const size = 2 + Math.sin(t + i) * 1.8;
      const colors = ["#f6d365", "#fda085", "#fbbf24", "#34d399", "#60a5fa", "#a78bfa", "#f472b6"];
      ctx.fillStyle = colors[i % colors.length];
      ctx.globalAlpha = 0.4 + Math.sin(t + i) * 0.25;
      ctx.beginPath();
      ctx.arc(cx, cy, size, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;

    // 顶部文字
    const textAlpha = 0.6 + Math.sin(frames * 0.03) * 0.3;
    ctx.fillStyle = "rgba(255,255,255," + textAlpha + ")";
    ctx.textAlign = "center";
    ctx.font = "700 16px system-ui";
    ctx.fillText("🏔️ 8848m 珠峰之巅 🏔️", canvas.width / 2, canvas.height / 3 - 50);
    ctx.textAlign = "start";

    // 定期放烟花
    if (frames % 90 === 0) {
      emitFirework(canvas.width * 0.3 + Math.random() * canvas.width * 0.4, canvas.height * 0.2 + Math.random() * canvas.height * 0.2);
    }
  }

  function drawHint() {
    if (!hintText || frames > hintUntil) return;
    const alpha = Math.min(1, (hintUntil - frames) / 15);
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.textAlign = "center";
    ctx.font = "700 18px system-ui";
    ctx.fillStyle = hintColor;
    ctx.fillText(hintText, canvas.width / 2, 44);
    ctx.restore();
  }

  function drawBadgeFlash() {
    if (badgeFlash <= 0) return;
    const alpha = (badgeFlash / 45) * 0.25;
    ctx.fillStyle = "rgba(255,255,255," + alpha + ")";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }

  // ─── 更新逻辑 ───────────────────────────────────────
  function update() {
    if (!paused) {
      frames += 1;

      if (!ended) {
        // 计时
        if (frames % 60 === 0) { gameTime += 1; timeEl.textContent = String(Math.floor(gameTime)); }

        // 段位闪烁衰减
        if (badgeFlash > 0) badgeFlash -= 1;

        // 跳跃缓冲和土狼时间
        if (jumpBufferFrames > 0) jumpBufferFrames -= 1;
        if (player.onPlatform) coyoteFrames = COYOTE_FRAMES;
        else if (coyoteFrames > 0) coyoteFrames -= 1;

        // ── 水平移动 ──
        let inputX = 0;
        if (keys.left && !keys.right) inputX = -1;
        if (keys.right && !keys.left) inputX = 1;
        const targetVx = inputX * MOVE_SPEED;
        const accel = player.onPlatform ? GROUND_ACCEL : AIR_ACCEL;
        player.vx += (targetVx - player.vx) * accel;
        if (inputX === 0) {
          player.vx *= player.onPlatform ? GROUND_FRICTION : AIR_DRAG;
        }
        if (Math.abs(player.vx) < 0.02) player.vx = 0;
        player.x += player.vx;
        if (player.vx < -0.08) player.dir = -1;
        if (player.vx > 0.08) player.dir = 1;
        if (player.x < 0) player.x = 0;
        if (player.x + PLAYER_W > canvas.width) player.x = canvas.width - PLAYER_W;

        const wasOnPlatform = player.onPlatform;

        // ── 垂直物理 ──
        player.vy += GRAVITY;
        if (player.vy > MAX_FALL) player.vy = MAX_FALL;
        player.y += player.vy;
        player.onPlatform = false;

        // ── 脆冰平台碎裂计时 ──
        for (let fi = 0; fi < platforms.length; fi++) {
          const fp = platforms[fi];
          if (fp.type === "fragile" && fp.crackTimer > 0) {
            fp.crackTimer -= 1;
            if (fp.crackTimer <= 0) {
              // 碎裂！移除平台并产生粒子
              emitBurst(fp.x + fp.w / 2, fp.y + fp.h / 2, 14, "#cbd5e1");
              platforms.splice(fi, 1);
              fi -= 1;
            }
          }
        }

        // ── 平台碰撞 ──
        for (let pi = 0; pi < platforms.length; pi++) {
          const pl = platforms[pi];
          if (
            player.vy >= 0 &&
            player.x + PLAYER_W > pl.x &&
            player.x < pl.x + pl.w &&
            player.y + PLAYER_H >= pl.y &&
            player.y + PLAYER_H <= pl.y + pl.h + player.vy + 5
          ) {
            const impactSpeed = player.vy;
            player.y = pl.y - PLAYER_H;
            player.vy = 0;
            player.onPlatform = true;

            if (!wasOnPlatform) {
              if (frames - lastLandFrame <= COMBO_WINDOW_FRAMES && pl.type !== "ground") {
                combo += 1;
              } else {
                combo = pl.type === "ground" ? 0 : 1;
              }
              comboEl.textContent = String(combo);
              lastLandFrame = frames;
              if (pl.type !== "ground") {
                const baseScore = 12 + Math.floor(height / 120);
                applyComboScore(baseScore);
              }
            }

            if (!wasOnPlatform && impactSpeed > 2.2) {
              cameraShake = Math.min(6, cameraShake + impactSpeed * 0.22);
              emitBurst(player.x + PLAYER_W / 2, player.y + PLAYER_H, 7, "#ffffff");
            }

            // 特殊平台效果
            if (pl.type === "bouncy") {
              executeJump(1.62);
              applyComboScore(18 + Math.floor(height / 180));
              cameraShake = Math.min(8, cameraShake + 1.2);
              emitBurst(player.x + PLAYER_W / 2, player.y + PLAYER_H, 10, "#ffd89a");
            }
            if (pl.type === "fragile" && pl.crackTimer === 0) {
              crackFragilePlatform(pl);
            }
            break;
          }
        }

        if (jumpBufferFrames > 0 && coyoteFrames > 0) {
          executeJump(1);
        }

        // 可变跳跃高度
        if (!jumpHeld && player.vy < 0) {
          player.vy *= 0.985;
        }

        // 侧边推开
        if (!player.onPlatform && player.vy >= -1) {
          for (let ci = 0; ci < platforms.length; ci++) {
            const cp = platforms[ci];
            if (
              player.y + PLAYER_H > cp.y && player.y < cp.y + cp.h &&
              player.x + PLAYER_W > cp.x && player.x < cp.x + cp.w
            ) {
              const ol = (player.x + PLAYER_W) - cp.x;
              const or2 = (cp.x + cp.w) - player.x;
              player.x = ol < or2 ? cp.x - PLAYER_W : cp.x + cp.w;
            }
          }
        }

        // ── 移动平台 ──
        for (let mi = 0; mi < platforms.length; mi++) {
          const mp = platforms[mi];
          if (mp.type === "moving" && mp.vx) {
            mp.x += mp.vx;
            // 平滑反弹
            if (mp.x < 0) { mp.x = 0; mp.vx = Math.abs(mp.vx); }
            if (mp.x + mp.w > canvas.width) { mp.x = canvas.width - mp.w; mp.vx = -Math.abs(mp.vx); }
            // 玩家搭乘
            if (
              player.onPlatform &&
              player.y + PLAYER_H >= mp.y - 1 &&
              player.y + PLAYER_H <= mp.y + 4 &&
              player.x + PLAYER_W > mp.x &&
              player.x < mp.x + mp.w
            ) {
              player.x += mp.vx;
            }
          }
        }

        // ── 上卷 ──
        if (player.y < SCROLL_THRESHOLD) {
          const scrollAmt = Math.min(SCROLL_THRESHOLD - player.y, 8);
          player.y += scrollAmt;
          height += HEIGHT_PER_SCROLL * scrollAmt;
          score += Math.floor(scrollAmt * (1 + combo * 0.05));
          heightEl.textContent = String(Math.round(height));
          scoreEl.textContent = String(score);
          updateBadge(height);

          for (let si = 0; si < platforms.length; si++) {
            platforms[si].y += scrollAmt;
          }

          if (height >= 8848 && !ended) {
            summitBanner.classList.add("show");
            if (summitCelebration === 0) {
              summitCelebration = 1;
              // 登顶时大爆发
              for (let fi = 0; fi < 5; fi++) {
                setTimeout(function () {
                  emitFirework(
                    canvas.width * 0.2 + Math.random() * canvas.width * 0.6,
                    canvas.height * 0.1 + Math.random() * canvas.height * 0.3
                  );
                }, fi * 200);
              }
              setHint("🎉 登顶珠峰！继续挑战更高分！", "#f6d365", 180);
            }
          }
        }

        // ── 生成新平台 ──
        // 在最高平台上方生成，保证可达
        let topY = canvas.height;
        for (let ti = 0; ti < platforms.length; ti++) {
          if (platforms[ti].y < topY) topY = platforms[ti].y;
        }
        if (topY > -60) {
          // 随高度增加间距，前期密集后期稀疏
          const difficultyGap = Math.max(0, (height - 500) / 7000);
          const minGap = 32 + difficultyGap * 28;
          const maxGap = 44 + difficultyGap * 40;
          const gap = minGap + Math.random() * (maxGap - minGap);
          addPlatform(topY - gap, false);
        }

        // ── 清理平台 ──
        // 移除远处出屏的平台（保留地面和碎裂中的脆冰平台）
        for (let ri = platforms.length - 1; ri >= 0; ri--) {
          const rp = platforms[ri];
          if (rp.type === "ground") continue;
          if (rp.type === "fragile" && rp.crackTimer > 0) continue;
          if (rp.y > canvas.height + 60) {
            platforms.splice(ri, 1);
          }
        }

        // 限制总数
        while (platforms.length > MAX_PLATFORMS) {
          for (let ri = platforms.length - 1; ri >= 0; ri--) {
            if (platforms[ri].type !== "ground" && platforms[ri].type !== "fragile") {
              platforms.splice(ri, 1);
              break;
            }
          }
        }

        // ── 掉落/救援/死亡 ──
        if (player.y > canvas.height + 60) {
          if (!rescueUsed) {
            rescueUsed = true;
            rescueEl.textContent = "救援：已使用";
            rescueEl.style.color = "#f97316";
            player.y = RESCUE_SAFE_Y;
            player.vy = JUMP_FORCE * 0.9;
            combo = 0;
            comboEl.textContent = "0";
            setHint("紧急救援启动！继续攀登", "#ffb703", 80);
            platforms.push({
              x: Math.max(0, Math.min(canvas.width - 140, player.x - 60)),
              y: player.y + PLAYER_H + 8,
              w: 140,
              h: PLATFORM_H,
              type: "normal",
              vx: 0,
              crackTimer: 0,
            });
            cameraShake = Math.max(cameraShake, 4);
          } else {
            ended = true;
            const rs = height >= 8848;
            if (rs) summitBanner.classList.add("show");
            submitScore(height, Math.floor(gameTime), rs).catch(function () {});
          }
        }

        // 粒子
        spawnSnow();
      }
    }

    updateParticles();

    // ── 绘制 ──
    if (cameraShake > 0.01) {
      cameraShake *= 0.85;
    } else {
      cameraShake = 0;
    }
    const shakeX = (Math.random() - 0.5) * cameraShake;
    const shakeY = (Math.random() - 0.5) * cameraShake;
    ctx.save();
    ctx.translate(shakeX, shakeY);
    drawBackground();
    drawPlatforms();
    drawParticles();
    drawPlayer();
    drawHint();
    drawBadgeFlash();
    ctx.restore();

    if (paused) {
      drawPauseOverlay();
    } else if (ended) {
      drawGameOver();
    } else if (height >= 8848) {
      drawSummit();
    }

    requestAnimationFrame(update);
  }

  // ─── 事件处理 ───────────────────────────────────────
  document.addEventListener("keydown", function (e) {
    if (e.code === "KeyP" || e.code === "Escape") {
      e.preventDefault();
      togglePause();
      return;
    }
    if (e.code === "Space") { e.preventDefault(); jump(); return; }
    if (e.code === "ArrowLeft" || e.code === "KeyA") { e.preventDefault(); keys.left = true; }
    if (e.code === "ArrowRight" || e.code === "KeyD") { e.preventDefault(); keys.right = true; }
  });

  document.addEventListener("keyup", function (e) {
    if (e.code === "Space") {
      jumpHeld = false;
      if (player.vy < 0) player.vy *= JUMP_CUT_MULT;
    }
    if (e.code === "ArrowLeft" || e.code === "KeyA") { keys.left = false; }
    if (e.code === "ArrowRight" || e.code === "KeyD") { keys.right = false; }
  });

  // 画布点击/触摸
  canvas.addEventListener("pointerdown", function (e) { e.preventDefault(); jump(); });
  canvas.addEventListener("pointerup", function () {
    jumpHeld = false;
    if (player.vy < 0) player.vy *= JUMP_CUT_MULT;
  });
  canvas.addEventListener("touchstart", function (e) { e.preventDefault(); jump(); });
  canvas.addEventListener("touchend", function () {
    jumpHeld = false;
    if (player.vy < 0) player.vy *= JUMP_CUT_MULT;
  });

  // 移动端方向按钮
  if (btnLeft) {
    btnLeft.addEventListener("pointerdown", function (e) { e.preventDefault(); keys.left = true; });
    btnLeft.addEventListener("pointerup", function (e) { e.preventDefault(); keys.left = false; });
    btnLeft.addEventListener("pointerleave", function (e) { keys.left = false; });
  }
  if (btnRight) {
    btnRight.addEventListener("pointerdown", function (e) { e.preventDefault(); keys.right = true; });
    btnRight.addEventListener("pointerup", function (e) { e.preventDefault(); keys.right = false; });
    btnRight.addEventListener("pointerleave", function (e) { keys.right = false; });
  }

  // 暂停按钮
  if (pauseBtn) {
    pauseBtn.addEventListener("click", togglePause);
  }

  restartBtn.addEventListener("click", resetGame);

  tabTop.addEventListener("click", function () {
    tabTop.classList.add("active"); tabSummit.classList.remove("active");
    activeTab = "top"; loadRank().catch(function () {});
  });

  tabSummit.addEventListener("click", function () {
    tabSummit.classList.add("active"); tabTop.classList.remove("active");
    activeTab = "summit"; loadRank().catch(function () {});
  });

  // ─── 启动 ───────────────────────────────────────────
  resetGame();
  loadRank().catch(function () {});
  requestAnimationFrame(update);
})();
