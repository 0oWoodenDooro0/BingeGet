/**
 * Standardized HTML5 Media Error and Preview Playback Definitions
 */

export const MediaErrorCode = {
  MEDIA_ERR_ABORTED: 1,
  MEDIA_ERR_NETWORK: 2,
  MEDIA_ERR_DECODE: 3,
  MEDIA_ERR_SRC_NOT_SUPPORTED: 4,
} as const;

export type MediaErrorCodeType = typeof MediaErrorCode[keyof typeof MediaErrorCode];

export const FALLBACK_HINT = '建議檢查本機解碼器或複製串流網址至外部播放器 (如 VLC / mpv) 播放';

export const MEDIA_ERROR_MESSAGES: Record<number, string> = {
  [MediaErrorCode.MEDIA_ERR_ABORTED]: '影片載入已中止',
  [MediaErrorCode.MEDIA_ERR_NETWORK]: `網路連線異常，無法載入影片串流。${FALLBACK_HINT}`,
  [MediaErrorCode.MEDIA_ERR_DECODE]: `影片解碼失敗，可能是系統缺少解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`,
  [MediaErrorCode.MEDIA_ERR_SRC_NOT_SUPPORTED]: `不支援的影片格式或系統缺少相應解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`,
};

export function getMediaErrorMessage(code: number): string {
  return MEDIA_ERROR_MESSAGES[code] || `多媒體播放發生未知錯誤 (錯誤碼: ${code})。${FALLBACK_HINT}`;
}
