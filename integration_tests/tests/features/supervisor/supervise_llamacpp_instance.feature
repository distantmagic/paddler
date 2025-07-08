Feature: Supervise llama.cpp instance

    @serial
    Scenario: supervisor starts and stops a llama.cpp instance
        Given fleet management is enabled
        Given balancer is running
        Given supervisor "supervisor-1" is running
        Given supervisor "supervisor-1" is registered
        Given supervisor "supervisor-1" uses model from Hugging Face (repo: "Mungert/all-MiniLM-L6-v2-GGUF", weights: "all-MiniLM-L6-v2-q4_0.gguf")
        Given agent "agent-1" is running (observes llama-server supervised by "supervisor-1")
        Given agent "agent-1" is registered
        Then llamacpp observed by "agent-1" uses model "all-MiniLM-L6-v2-q4_0.gguf"
