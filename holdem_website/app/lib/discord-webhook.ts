export async function sendToDiscordWebhook(data: {
  email: string
  name?: string
  message?: string
}) {
  const webhookUrl = 'https://discordapp.com/api/webhooks/1368904320590020648/4pVpMw4_gkVppZ6wjBub4pRKrk9H7OHVJNAc9exFeCCYzcC3rt6LOd73EOU_nvF052qf'

  const content = [
    `**New Contact Form Submission**`,
    `**Email:** ${data.email}`,
    data.name && `**Name:** ${data.name}`,
    data.message && `**Message:**\n${data.message}`,
  ]
    .filter(Boolean)
    .join('\n')

  try {
    const response = await fetch(webhookUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        content,
        username: 'Contact Form',
        avatar_url: 'https://iamzub.in/favicon.ico',
      }),
    })

    if (!response.ok) {
      const errorData = await response.json()
      console.error('Discord API Error:', errorData)
      throw new Error(`Failed to send message to Discord: ${errorData.message}`)
    }

    return true
  } catch (error) {
    console.error('Error sending to Discord:', error)
    return false
  }
} 