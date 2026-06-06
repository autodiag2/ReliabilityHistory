const { invoke } = window.__TAURI__.core;

async function loadDays() {
  const days = await invoke("load_days");
  updateDays(days);
  renderChart(days);
  updateScore(days);
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#refresh-button").addEventListener("click", loadDays);
  loadDays();
});

function updateDays(days) {
  const timeline = document.getElementById("timeline");
  timeline.innerHTML = "";

  days.forEach((day, index) => {
    const button = document.createElement("button");
    button.className = "day-item";
    button.innerHTML = `<strong>${day.day}</strong><span>${day.score.toFixed(1)}</span>`;
    button.addEventListener("click", () => showEvents(day.events));
    timeline.appendChild(button);
  });

  if (days.length > 0) {
    showEvents(days[days.length - 1].events);
  }
}

function renderChart(days) {
  const chart = document.getElementById("chart");
  chart.innerHTML = "";

  days.forEach((day) => {
    const bar = document.createElement("div");
    bar.className = "chart-bar";
    const height = Math.max(10, day.score * 8);
    bar.style.height = `${height}px`;
    bar.title = `${day.day}: ${day.score.toFixed(1)}`;
    chart.appendChild(bar);
  });
}

function updateScore(days) {
  const currentScore = document.getElementById("current-score");
  if (days.length > 0) {
    currentScore.textContent = `${days[days.length - 1].score.toFixed(1)} / 10`;
  } else {
    currentScore.textContent = "No data";
  }
}

function showEvents(events) {
  const body = document.querySelector("#events tbody");
  body.innerHTML = "";

  events.forEach((ev) => {
    const row = document.createElement("tr");
    row.className = "event-row";
    row.innerHTML = `
      <td>${new Date(ev.timestamp).toLocaleString()}</td>
      <td>${ev.application}</td>
      <td>${ev.reason}</td>
    `;
    row.addEventListener("click", () => {
      window.alert(`Source: ${ev.application}\nTime: ${new Date(ev.timestamp).toLocaleString()}\nReason: ${ev.reason}`);
    });
    body.appendChild(row);
  });
}
