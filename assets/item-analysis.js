(() => {
  const form = document.getElementById("item-photo-analysis-form");
  if (!form) {
    return;
  }

  const input = document.getElementById("item-photo-analysis-input");
  const status = document.getElementById("item-photo-analysis-status");
  const warnings = document.getElementById("item-photo-analysis-warnings");
  const endpoint = form.dataset.analysisEndpoint;

  const fieldMap = {
    name: "input[name='name']",
    category: "input[name='category']",
    subcategory: "input[name='subcategory']",
    brand: "input[name='brand']",
    size: "input[name='size']",
    color_primary: "input[name='color_primary']",
    color_secondary: "input[name='color_secondary']",
    material: "input[name='material']",
    season: "input[name='season']",
    formality: "input[name='formality']",
    status: "input[name='status']",
    notes: "textarea[name='notes']",
  };

  function updateWarnings(items) {
    warnings.innerHTML = "";
    if (!Array.isArray(items) || items.length === 0) {
      return;
    }

    items.forEach((warning) => {
      const li = document.createElement("li");
      li.textContent = warning;
      warnings.appendChild(li);
    });
  }

  function applySuggestion(suggestion) {
    Object.entries(fieldMap).forEach(([key, selector]) => {
      const value = suggestion[key];
      if (typeof value !== "string" || value.trim() === "") {
        return;
      }

      const element = document.querySelector(selector);
      if (element) {
        element.value = value;
      }
    });
  }

  async function runAnalysis() {
    if (!endpoint) {
      status.textContent = "Photo analysis endpoint is not configured.";
      return;
    }

    const file = input.files && input.files[0];
    if (!file) {
      status.textContent = "Choose an image first.";
      updateWarnings([]);
      return;
    }

    const body = new FormData();
    body.append("file", file);

    status.textContent = "Analyzing photo with Codex...";
    updateWarnings([]);

    try {
      const response = await fetch(endpoint, {
        method: "POST",
        body,
      });

      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const message =
          payload &&
          payload.error &&
          typeof payload.error.message === "string"
            ? payload.error.message
            : `Photo analysis failed with HTTP ${response.status}.`;
        throw new Error(message);
      }

      const suggestion = payload && payload.suggestion ? payload.suggestion : null;
      if (!suggestion) {
        throw new Error("Photo analysis did not return a suggestion payload.");
      }

      applySuggestion(suggestion);
      status.textContent = suggestion.summary || "Photo analysis completed.";
      updateWarnings(suggestion.warnings || []);
    } catch (error) {
      status.textContent =
        error instanceof Error ? error.message : "Photo analysis failed.";
      updateWarnings([]);
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    void runAnalysis();
  });

  input.addEventListener("change", () => {
    if (input.files && input.files.length > 0) {
      void runAnalysis();
    }
  });
})();
