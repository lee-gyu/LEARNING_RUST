// @ts-check
import fs from 'node:fs';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { sourceRoots, syncContent } from './scripts/sync-content.mjs';
import { base, repoUrl, site, siteOptions } from './site.config.mjs';

// 저장소의 README와 소스 파일을 src/content/docs/ 로 옮긴 뒤 사이트를 구성합니다.
const { sidebar } = syncContent(siteOptions);

export default defineConfig({
	site,
	base,
	integrations: [
		watchSources(),
		starlight({
			title: 'LEARNING_RUST',
			favicon: 'https://lee-gyu.github.io/favicon.ico',
			locales: { root: { label: '한국어', lang: 'ko' } },
			social: [{ icon: 'github', label: 'GitHub', href: repoUrl }],
			sidebar,
		}),
	],
});

/**
 * dev 서버가 떠 있는 동안 원본(README, 예제 파일)이 바뀌면 다시 동기화합니다.
 * @returns {import('astro').AstroIntegration}
 */
function watchSources() {
	/** @type {fs.FSWatcher[]} */
	let watchers = [];
	return {
		name: 'watch-sources',
		hooks: {
			'astro:server:setup': ({ logger }) => {
				/** @type {NodeJS.Timeout | undefined} */
				let timer;
				const resync = () => {
					clearTimeout(timer);
					timer = setTimeout(() => {
						const result = syncContent(siteOptions);
						if (result.written || result.removed) {
							logger.info(`콘텐츠 동기화: ${result.written}개 갱신, ${result.removed}개 삭제`);
						}
						if (JSON.stringify(result.sidebar) !== JSON.stringify(sidebar)) {
							logger.warn('사이드바 구성이 바뀌었습니다. dev 서버를 다시 시작하세요.');
						}
					}, 150);
				};
				const { files, dirs } = sourceRoots();
				watchers = [
					...files.map((file) => fs.watch(file, resync)),
					...dirs.map((dir) =>
						fs.watch(dir, { recursive: true }, (_event, name) => {
							if (!name || !/(^|[\\/])target[\\/]/.test(name)) resync();
						}),
					),
				];
			},
			'astro:server:done': () => {
				for (const watcher of watchers) watcher.close();
				watchers = [];
			},
		},
	};
}
