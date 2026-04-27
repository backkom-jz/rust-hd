(function () {
  var topics = document.querySelectorAll(".redis-topic");
  var panels = document.querySelectorAll(".redis-panel");
  var replays = document.querySelectorAll(".redis-replay");

  function showTopic(id) {
    topics.forEach(function (btn) {
      btn.setAttribute("aria-selected", btn.getAttribute("data-topic") === id);
    });
    panels.forEach(function (p) {
      p.classList.remove("playing");
      var on = p.getAttribute("data-type") === id;
      p.classList.toggle("is-active", on);
    });
    var active = document.querySelector('.redis-panel[data-type="' + id + '"]');
    if (active) {
      void active.offsetWidth;
      active.classList.add("playing");
    }
  }

  topics.forEach(function (btn) {
    btn.addEventListener("click", function () {
      showTopic(btn.getAttribute("data-topic"));
    });
  });

  replays.forEach(function (btn) {
    btn.addEventListener("click", function () {
      var panel = btn.closest(".redis-panel");
      if (!panel) return;
      panel.classList.remove("playing");
      void panel.offsetWidth;
      panel.classList.add("playing");
    });
  });

  var first = document.querySelector(".redis-topic[aria-selected='true']");
  if (first) {
    showTopic(first.getAttribute("data-topic"));
  }
})();
