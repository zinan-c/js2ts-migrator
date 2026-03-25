const dropzone = document.getElementById("dropzone");
const fileInput = document.getElementById("fileInput");
const output = document.getElementById("output");
const downloadBtn = document.getElementById("downloadBtn");
const clearBtn = document.getElementById("clearBtn");
const meta = document.getElementById("meta");

let currentFileName = "output.ts";
let currentContent = "";

function setButtonsEnabled(enabled) {
  downloadBtn.disabled = !enabled;
  clearBtn.disabled = !enabled;
}

async function handleFile(file) {
  const formData = new FormData();
  formData.append("file", file, file.name);

  output.value = "Migrating...";
  meta.textContent = "";
  setButtonsEnabled(false);

  try {
    const res = await fetch("/api/migrate", {
      method: "POST",
      body: formData,
    });
    const data = await res.json();
    if (!res.ok) {
      throw new Error(data.error || "Failed to migrate file");
    }
    currentContent = data.content || "";
    output.value = currentContent;
    currentFileName = data.output_name || "output.ts";
    meta.textContent = `Vars: ${data.var_count} | Lines: ${data.line_count} | Output: ${currentFileName}`;
    setButtonsEnabled(true);
  } catch (err) {
    output.value = "";
    meta.textContent = `Error: ${err.message || err}`;
  }
}

dropzone.addEventListener("click", () => fileInput.click());
fileInput.addEventListener("change", (e) => {
  const file = e.target.files && e.target.files[0];
  if (file) handleFile(file);
});

dropzone.addEventListener("dragover", (e) => {
  e.preventDefault();
  dropzone.classList.add("dragover");
});
dropzone.addEventListener("dragleave", () => dropzone.classList.remove("dragover"));
dropzone.addEventListener("drop", (e) => {
  e.preventDefault();
  dropzone.classList.remove("dragover");
  const file = e.dataTransfer.files && e.dataTransfer.files[0];
  if (file) handleFile(file);
});

downloadBtn.addEventListener("click", () => {
  const blob = new Blob([currentContent], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = currentFileName || "output.ts";
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
});

clearBtn.addEventListener("click", () => {
  output.value = "";
  meta.textContent = "";
  currentContent = "";
  currentFileName = "output.ts";
  fileInput.value = "";
  setButtonsEnabled(false);
});
