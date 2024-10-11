async function parseGitOid(bytes: Uint8Array): Promise<string> {
  // Create the header
  const header = new TextEncoder().encode(`blob ${bytes.length}\0`);

  // Concatenate the header and the original file content
  const combined = new Uint8Array(header.length + bytes.length);
  combined.set(header);
  combined.set(bytes, header.length);

  // Compute the SHA-1 hash
  const hashBuffer = await crypto.subtle.digest("SHA-1", combined);
  const hashArray = Array.from(new Uint8Array(hashBuffer));

  // Convert the hash to a hexadecimal string
  return hashArray.map(b => b.toString(16).padStart(2, "0")).join("");
}

function base64String(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (event: ProgressEvent<FileReader>) => {
      if (event.target?.result && typeof event.target.result === "string") {
        resolve(event.target.result);
      } else {
        reject(new Error("Failed to generate base64 string"));
      }
    };

    reader.readAsDataURL(file);
  });
}

export async function embed(file: File) {
  const bytes = new Uint8Array(await file.arrayBuffer());
  const oid = await parseGitOid(bytes);
  const content = await base64String(file);
  return { oid, name: file.name, content };
}
