name: "🔒 Security Vulnerability"
description: "Report a security vulnerability responsibly."
labels: ["security", "urgent"]
body:
  - type: markdown
    attributes:
      value: |
        ## 🚨 Security Notice
        Thank you for reporting a security issue. **Do NOT disclose sensitive details publicly**.  
        Instead, please email us at **[security@yourdomain.com](mailto:security@yourdomain.com)** if necessary.

  - type: checkboxes
    id: confidentiality
    attributes:
      label: "⚠️ Confidentiality Agreement"
      description: "By submitting this issue, you confirm that you will not disclose this vulnerability publicly until an official fix is released."
      options:
        - label: "I agree to report this responsibly and will not share sensitive details publicly."
          required: true

  - type: textarea
    id: vulnerability-description
    attributes:
      label: "🛑 Describe the vulnerability"
      description: "Provide a detailed description of the security vulnerability."
      placeholder: "Explain what happens, how an attacker could exploit this, and potential impact."
      value: ""

  - type: textarea
    id: reproduction-steps
    attributes:
      label: "📌 Steps to Reproduce"
      description: "Provide step-by-step instructions to reproduce the issue."
      placeholder: |
        1. Go to '...'
        2. Click on '...'
        3. Enter '...' in the form
        4. See error message '...'"
      value: ""

  - type: textarea
    id: expected-behavior
    attributes:
      label: "✅ Expected Behavior"
      description: "Describe what should happen instead."
      placeholder: "The application should prevent this action by ..."

  - type: textarea
    id: system-info
    attributes:
      label: "🖥 System Information"
      description: "Provide details about the environment where this issue occurs."
      placeholder: |
        - OS: [e.g. Windows 11, Ubuntu 22.04]
        - Browser: [e.g. Chrome 98, Firefox 99]
        - Affected Version: [e.g. v1.2.3]

  - type: textarea
    id: additional-info
    attributes:
      label: "📝 Additional Information (Optional)"
      description: "Provide any extra details, logs, or screenshots that could help us investigate."

  - type: markdown
    attributes:
      value: |
        ---
        **🔒 If the issue is critical and requires urgent attention, please email: [security@yourdomain.com](mailto:security@yourdomain.com).**  
        We appreciate your responsible disclosure! 🙌
