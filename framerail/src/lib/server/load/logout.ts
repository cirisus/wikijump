import defaults from "$lib/defaults"
import { parseAcceptLangHeader } from "$lib/locales"
import { translate } from "$lib/server/deepwell/translate"
import type { TranslateKeys } from "$lib/types"

export async function loadLogoutPage(request, cookies) {
  // Set up parameters
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  let locales = parseAcceptLangHeader(request)

  let viewData: Record<string, any> = {
    isLoggedIn: Boolean(sessionToken)
  }

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  const translateKeys: TranslateKeys = {
    ...defaults.translateKeys,

    // Page actions
    "cancel": {},
    "logout": {},

    // misc
    "logout.toast": {}
  }

  const translated = await translate(locales, translateKeys)

  viewData.internationalization = translated

  // Return to page for rendering
  return viewData
}
