const { invoke } = window.__TAURI__.core;

async function loadDays() {
  const days = await invoke("load_days");
  renderChart(days);
  updateScore(days);
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#refresh-button").addEventListener("click", loadDays);
  loadDays();
});
function selectDay(index, days) {
  const day = days[index];
  if (day) {
    showEvents(day.events);
  }
}
const AUTORECOVER_SCORE_PER_DAY = 1.0;
function renderChart(days) {
  const chart = document.getElementById("chart");

  const width = chart.clientWidth || 1200;
  const height = 340;

  chart.innerHTML = "";

  const svg =
    document.createElementNS(
      "http://www.w3.org/2000/svg",
      "svg"
    );

  svg.setAttribute("width", width);
  svg.setAttribute("height", height);

  const graphHeight = 180;
  const dayWidth = width / Math.max(days.length, 1);

  let score = 10.0;

  const points = [];

  days.forEach((day, dayIndex) => {

    if (dayIndex > 0) {
      score = Math.min(10.0, score + AUTORECOVER_SCORE_PER_DAY);
    }

    points.push([
      dayIndex,
      score
    ]);

    const events =
      [...day.events].sort(
        (a, b) =>
          new Date(a.timestamp)
          - new Date(b.timestamp)
      );

    events.forEach((event) => {

      const ts =
        new Date(event.timestamp);

      const fraction =
        (
          ts.getHours()
          + ts.getMinutes() / 60
        ) / 24;

      score -=
        event.kind === "ApplicationFailure"
          ? 1.0
        : event.kind === "SystemFailure"
          ? 1.5
        : event.kind === "Warning"
          ? 0.25
        : 0.0;

      score =
        Math.max(0.0, score);

      points.push([
        dayIndex + fraction,
        score
      ]);
    });

    score =
      Math.min(10.0, score + 0.05);
  });

  for (let i = 0; i <= 10; i++) {

    const y =
      graphHeight * i / 10;

    const line =
      document.createElementNS(
        "http://www.w3.org/2000/svg",
        "line"
      );

    line.setAttribute("x1", 0);
    line.setAttribute("y1", y);
    line.setAttribute("x2", width);
    line.setAttribute("y2", y);
    line.setAttribute("stroke", "#ddd");

    svg.appendChild(line);
  }

  days.forEach((day, i) => {

    const x = i * dayWidth;

    const rect =
      document.createElementNS(
        "http://www.w3.org/2000/svg",
        "rect"
      );

    rect.setAttribute("x", x);
    rect.setAttribute("y", 0);
    rect.setAttribute("width", dayWidth);
    rect.setAttribute("height", height);

    rect.setAttribute(
      "fill",
      i === window.selectedDay
        ? "#dbeeff"
        : "transparent"
    );

    rect.style.cursor = "pointer";

    rect.addEventListener(
      "click",
      () => {
        window.selectedDay = i;
        selectDay(i, days);
        renderChart(days);
      }
    );

    svg.appendChild(rect);

    const grid =
      document.createElementNS(
        "http://www.w3.org/2000/svg",
        "line"
      );

    grid.setAttribute("x1", x);
    grid.setAttribute("y1", 0);
    grid.setAttribute("x2", x);
    grid.setAttribute("y2", height);

    grid.setAttribute(
      "stroke",
      "#e0e0e0"
    );

    svg.appendChild(grid);
  });

  let path = "";

  points.forEach(([xv, score], index) => {

    const x =
      xv * dayWidth;

    const y =
      graphHeight
      * (1 - score / 10);

    path +=
      (index === 0 ? "M" : "L")
      + x
      + " "
      + y
      + " ";
  });

  const curve =
    document.createElementNS(
      "http://www.w3.org/2000/svg",
      "path"
    );

  curve.setAttribute("d", path);

  curve.setAttribute(
    "stroke",
    "#2d89ef"
  );

  curve.setAttribute(
    "stroke-width",
    "2"
  );

  curve.setAttribute(
    "fill",
    "none"
  );

  svg.appendChild(curve);

  const rowY = {
    Information: graphHeight + 35,
    Warning: graphHeight + 65,
    ApplicationFailure: graphHeight + 95,
    SystemFailure: graphHeight + 95
  };

  const rowIcon = {
    Information: "ℹ️",
    Warning: "⚠️",
    ApplicationFailure: "❌",
    SystemFailure: "❌"
  };

  days.forEach((day, dayIndex) => {

    day.events.forEach(event => {

      const ts =
        new Date(event.timestamp);

      const fraction =
        (
          ts.getHours()
          + ts.getMinutes() / 60
        ) / 24;

      const x =
        dayIndex * dayWidth
        + fraction * dayWidth;

      const y =
        rowY[event.kind]
        || rowY.Information;

      const text =
        document.createElementNS(
          "http://www.w3.org/2000/svg",
          "text"
        );

      text.setAttribute("x", x);
      text.setAttribute("y", y);

      text.setAttribute(
        "text-anchor",
        "middle"
      );

      text.textContent =
        rowIcon[event.kind]
        || "ℹ️";

      svg.appendChild(text);
    });
  });

  chart.appendChild(svg);
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
    const icon = ev.kind === "ApplicationFailure"
      ? "❌"
      : ev.kind === "SystemFailure"
        ? "❌"
        : ev.kind === "Warning"
          ? "⚠️"
          : "ℹ️";
    row.innerHTML = `
      <td>${icon}</td>
      <td>${new Date(ev.timestamp).toLocaleTimeString()}</td>
      <td>${ev.application}</td>
      <td>${ev.reason}</td>
    `;
    row.addEventListener("click", () => {
      window.alert(`Source: ${ev.application}\nTime: ${new Date(ev.timestamp).toLocaleTimeString()}\nReason: ${ev.reason}`);
    });
    body.appendChild(row);
  });
}
