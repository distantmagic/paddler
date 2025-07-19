export function urlToAgentDesiredState(url) {
    if (url.hostname === "huggingface.co") {
        const parts = url.pathname.split("/");
        const filename = parts.pop();
        if (!filename) {
            throw new Error("Invalid Hugging Face URL: No filename found");
        }
        const repo = parts.slice(1, 3).join("/");
        return {
            model: {
                HuggingFace: {
                    filename,
                    repo,
                },
            },
        };
    }
    else if (url.protocol === "file:") {
        return {
            model: {
                Local: url.pathname,
            },
        };
    }
    else {
        throw new Error("Unsupported URL format");
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXJsVG9BZ2VudERlc2lyZWRTdGF0ZS5qcyIsInNvdXJjZVJvb3QiOiIvaG9tZS9tY2hhcnl0b25pdWsvd29ya3NwYWNlL2ludGVudGVlL3BhZGRsZXIvIiwic291cmNlcyI6WyJyZXNvdXJjZXMvdHMvdXJsVG9BZ2VudERlc2lyZWRTdGF0ZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFFQSxNQUFNLFVBQVUsc0JBQXNCLENBQUMsR0FBUTtJQUM3QyxJQUFJLEdBQUcsQ0FBQyxRQUFRLEtBQUssZ0JBQWdCLEVBQUUsQ0FBQztRQUN0QyxNQUFNLEtBQUssR0FBRyxHQUFHLENBQUMsUUFBUSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQztRQUN0QyxNQUFNLFFBQVEsR0FBRyxLQUFLLENBQUMsR0FBRyxFQUFFLENBQUM7UUFFN0IsSUFBSSxDQUFDLFFBQVEsRUFBRSxDQUFDO1lBQ2QsTUFBTSxJQUFJLEtBQUssQ0FBQyw2Q0FBNkMsQ0FBQyxDQUFDO1FBQ2pFLENBQUM7UUFFRCxNQUFNLElBQUksR0FBRyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUM7UUFFekMsT0FBTztZQUNMLEtBQUssRUFBRTtnQkFDTCxXQUFXLEVBQUU7b0JBQ1gsUUFBUTtvQkFDUixJQUFJO2lCQUNMO2FBQ0Y7U0FDRixDQUFDO0lBQ0osQ0FBQztTQUFNLElBQUksR0FBRyxDQUFDLFFBQVEsS0FBSyxPQUFPLEVBQUUsQ0FBQztRQUNwQyxPQUFPO1lBQ0wsS0FBSyxFQUFFO2dCQUNMLEtBQUssRUFBRSxHQUFHLENBQUMsUUFBUTthQUNwQjtTQUNGLENBQUM7SUFDSixDQUFDO1NBQU0sQ0FBQztRQUNOLE1BQU0sSUFBSSxLQUFLLENBQUMsd0JBQXdCLENBQUMsQ0FBQztJQUM1QyxDQUFDO0FBQ0gsQ0FBQyJ9