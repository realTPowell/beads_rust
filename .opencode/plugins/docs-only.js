export const DocsOnlyPlugin = async ({ directory }) => {
  const docsPath = directory + "/docs/"
  return {
    "tool.execute.before": async (input, output) => {
      const tool = input.tool
      if (tool === "write" || tool === "edit") {
        const filePath = output.args.filePath
        if (filePath && !filePath.startsWith(docsPath)) {
          throw new Error(
            `Blocked: writes are restricted to ./docs/ (attempted: ${filePath})`
          )
        }
      }
    },
  }
}
