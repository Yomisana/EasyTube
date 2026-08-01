import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import opencc from 'opencc-js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const localesDir = join(__dirname, '..', 'src', 'i18n', 'messages');

const twSource = JSON.parse(
  readFileSync(join(localesDir, 'zh-TW.json'), 'utf-8'),
);

const converter = opencc.Converter({ from: 'twp', to: 'cn' });

function deepConvert(obj: unknown): unknown {
  if (typeof obj === 'string') {
    return converter(obj);
  }
  if (Array.isArray(obj)) {
    return obj.map(deepConvert);
  }
  if (obj !== null && typeof obj === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = deepConvert(value);
    }
    return result;
  }
  return obj;
}

const zhCn = deepConvert(twSource);

writeFileSync(
  join(localesDir, 'zh-CN.json'),
  `${JSON.stringify(zhCn, null, 2)}\n`,
  'utf-8',
);

console.log('Generated zh-CN.json from zh-TW.json (OpenCC tw2sp)');
