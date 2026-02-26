import { generate_html, generate_pdf } from "$lib/converter/pkg/converter";

export enum ExportTypes {
  Fountain,
  Pdf,
  Html,
}

export function exportToFile(script: string, type: ExportTypes) {
  let blob;
  let fileExtension;
  switch (type) {
    case ExportTypes.Fountain:
      blob = new Blob([script], { type: 'text/plain' });
      fileExtension = "fountain";
      break;
    case ExportTypes.Html:
      const html = generate_html(script);
      blob = new Blob([html], { type: 'text/html' });
      fileExtension = "html";
      break;
    case ExportTypes.Pdf:
      const pdf = generate_pdf(script);
      blob = new Blob([pdf as any], { type: 'application/pdf' });
      fileExtension = "pdf";
      break;
    default:
      console.log("Unsupported export type");
      return;
  }
  // TODO: when multiple files exist, filename should be script name
  const filename = `script.${fileExtension}`;

  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(url);
  document.body.removeChild(a);
}
