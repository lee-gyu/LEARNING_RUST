// 저장소의 README.md와 소스 파일을 Starlight 콘텐츠(src/content/docs/)로 변환합니다.
//
// 원본은 저장소 루트의 파트/챕터 디렉터리에 그대로 두고, 이 스크립트가 빌드할 때마다
// 사이트용 사본을 만듭니다. src/content/docs/ 는 전부 생성물이므로 직접 고치지 않습니다.
//
//   README.md                      → /
//   docs/roadmap.md                → /roadmap/
//   PROGRESS.md                    → /progress/
//   partN-x/README.md              → /partN-x/
//   partN-x/chM-y/README.md        → /partN-x/chM-y/
//   partN-x/chM-y/examples/a.rs    → /partN-x/chM-y/examples/a-rs/   (코드 페이지)
//   partN-x/chM-y/examples/        → /partN-x/chM-y/examples/        (파일 목록 페이지)

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const WEB_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REPO_DIR = path.resolve(WEB_DIR, '..');
const OUT_DIR = path.join(WEB_DIR, 'src/content/docs');

/** 파트/챕터 밖에 있는 문서: 원본 경로 → 사이트 slug */
const ROOT_DOCS = {
	'README.md': '',
	'docs/roadmap.md': 'roadmap',
	'PROGRESS.md': 'progress',
};

/** 코드 페이지로 만드는 파일: 확장자 → 코드 블록 언어 */
const CODE_LANGS = {
	rs: 'rust',
	toml: 'toml',
	lock: 'toml',
	txt: 'text',
	md: 'markdown',
	sh: 'sh',
	json: 'json',
	yaml: 'yaml',
	yml: 'yaml',
};

const SKIP_DIRS = new Set(['target', 'node_modules']);

/**
 * @param {{ base: string, repoUrl: string, branch: string }} options
 *   base: 사이트의 기준 경로(예: '/LEARNING_RUST'), repoUrl·branch: 페이지가 없는 파일의 링크 대상
 * @returns {{ sidebar: Sidebar, written: number, removed: number }}
 * @typedef {NonNullable<import('@astrojs/starlight/types').StarlightUserConfig['sidebar']>} Sidebar
 */
export function syncContent({ base, repoUrl, branch }) {
	const site = collect();
	const urlOf = (slug) => `${base.replace(/\/$/, '')}/${slug ? slug + '/' : ''}`;
	const githubUrl = (rel, kind) => `${repoUrl}/${kind}/${branch}/${rel}`;

	/** 원본 문서 안의 상대 링크 하나를 사이트 URL로 바꿉니다. */
	const resolveLink = (target, fromRel) => {
		if (/^([a-z][a-z0-9+.-]*:|#|\/)/i.test(target)) return target;
		const hashAt = target.indexOf('#');
		const file = hashAt < 0 ? target : target.slice(0, hashAt);
		const hash = hashAt < 0 ? '' : target.slice(hashAt);
		const rel = path.posix
			.normalize(path.posix.join(path.posix.dirname(fromRel), decodeURI(file)))
			.replace(/\/$/, '');
		if (rel.startsWith('..')) return target;

		const key = rel === '.' ? '' : rel;
		const slug =
			site.docs.get(key)?.slug ??
			site.docs.get(key ? `${key}/README.md` : 'README.md')?.slug ??
			site.pages.get(key);
		if (slug !== undefined) return urlOf(slug) + hash;

		const abs = path.join(REPO_DIR, key);
		if (!fs.existsSync(abs)) {
			console.warn(`[sync-content] ${fromRel}: 링크 대상이 없습니다 → ${target}`);
			return target;
		}
		return githubUrl(key, fs.statSync(abs).isDirectory() ? 'tree' : 'blob') + hash;
	};

	/** @type {Map<string, string>} 출력 파일(OUT_DIR 기준) → 내용 */
	const output = new Map();

	for (const [rel, doc] of site.docs) {
		const frontmatter = [`title: ${JSON.stringify(doc.title)}`];
		if (doc.sidebarLabel) frontmatter.push(`sidebar:\n  label: ${JSON.stringify(doc.sidebarLabel)}`);
		const body = rewriteLinks(doc.body, (target) => resolveLink(target, rel));
		output.set(slugToFile(doc.slug, true), page(rel, frontmatter, body));
	}

	for (const chapter of site.chapters) {
		const back = `[← ${chapter.title}](${urlOf(chapter.slug)})`;

		for (const file of chapter.files) {
			const source = fs.readFileSync(path.join(REPO_DIR, chapter.dir, file.rel), 'utf8');
			const fence = '`'.repeat(Math.max(3, longestBacktickRun(source) + 1));
			const frontmatter = [
				`title: ${JSON.stringify(`${chapter.name}/${file.rel}`)}`,
				'tableOfContents: false',
			];
			const body = [
				`${back} · [GitHub에서 보기](${githubUrl(`${chapter.dir}/${file.rel}`, 'blob')})`,
				'',
				`${fence}${file.lang} title=${JSON.stringify(file.rel)}`,
				source.replace(/\n$/, ''),
				fence,
				'',
			].join('\n');
			output.set(slugToFile(file.slug, false), page(`${chapter.dir}/${file.rel}`, frontmatter, body));
		}

		for (const dir of chapter.dirs) {
			const frontmatter = [
				`title: ${JSON.stringify(`${chapter.name}/${dir.rel}/`)}`,
				'tableOfContents: false',
			];
			const items = chapter.files
				.filter((file) => file.rel.startsWith(dir.rel + '/'))
				.map((file) => `- [\`${file.rel.slice(dir.rel.length + 1)}\`](${urlOf(file.slug)})`);
			const body = [
				`${back} · [GitHub에서 보기](${githubUrl(`${chapter.dir}/${dir.rel}`, 'tree')})`,
				'',
				...items,
				'',
			].join('\n');
			output.set(slugToFile(dir.slug, true), page(`${chapter.dir}/${dir.rel}/`, frontmatter, body));
		}
	}

	return { sidebar: site.sidebar, ...writeOutput(output) };
}

/** 원본 디렉터리 목록: 변경을 감시할 때 씁니다. */
export function sourceRoots() {
	return {
		files: Object.keys(ROOT_DOCS).map((rel) => path.join(REPO_DIR, rel)),
		dirs: partDirs().map((dir) => path.join(REPO_DIR, dir)),
	};
}

// ---------------------------------------------------------------------------
// 수집

function collect() {
	/** @type {Map<string, { slug: string, title: string, body: string, sidebarLabel?: string }>} 원본 경로 → 문서 */
	const docs = new Map();
	/** @type {Map<string, string>} 원본 경로(파일·디렉터리) → 코드/목록 페이지 slug */
	const pages = new Map();
	const chapters = [];
	const sidebar = [];

	const guide = [];
	for (const [rel, slug] of Object.entries(ROOT_DOCS)) {
		if (!addDoc(docs, rel, slug)) continue;
		guide.push(slug === '' ? { label: '소개', slug: 'index' } : slug);
	}
	if (guide.length) sidebar.push({ label: '안내', items: guide });

	for (const part of partDirs()) {
		const partDoc = addDoc(docs, `${part}/README.md`, part, { sidebarLabel: '개요' });
		const items = partDoc ? [part] : [];

		for (const name of numbered(path.join(REPO_DIR, part), /^ch(\d+)-/)) {
			const dir = `${part}/${name}`;
			const doc = addDoc(docs, `${dir}/README.md`, dir);
			if (!doc) continue; // 아직 README가 없는 챕터(작성 중)는 건너뜁니다.

			const chapter = { name, dir, slug: dir, title: doc.title, files: [], dirs: [] };
			walk(path.join(REPO_DIR, dir), '', chapter);
			for (const file of chapter.files) pages.set(`${dir}/${file.rel}`, file.slug);
			for (const sub of chapter.dirs) pages.set(`${dir}/${sub.rel}`, sub.slug);
			chapters.push(chapter);
			items.push(dir);
		}

		if (items.length) sidebar.push({ label: partDoc?.title ?? part, items });
	}

	return { docs, pages, chapters, sidebar };
}

function addDoc(docs, rel, slug, extra = {}) {
	const abs = path.join(REPO_DIR, rel);
	if (!fs.existsSync(abs)) return undefined;
	const { title, body } = splitTitle(fs.readFileSync(abs, 'utf8'), path.basename(path.dirname(abs)));
	const doc = { slug, title, body, ...extra };
	docs.set(rel, doc);
	return doc;
}

/** 챕터 디렉터리를 훑어 코드 페이지로 만들 파일과, 파일이 들어 있는 하위 디렉터리를 모읍니다. */
function walk(absDir, rel, chapter) {
	let found = false;
	const entries = fs.readdirSync(absDir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name));
	for (const entry of entries) {
		if (entry.name.startsWith('.')) continue;
		const childRel = rel ? `${rel}/${entry.name}` : entry.name;
		if (entry.isDirectory()) {
			if (SKIP_DIRS.has(entry.name)) continue;
			const index = chapter.dirs.push({ rel: childRel, slug: `${chapter.slug}/${slugify(childRel)}` }) - 1;
			if (walk(path.join(absDir, entry.name), childRel, chapter)) found = true;
			else chapter.dirs.splice(index, 1);
		} else if (entry.isFile()) {
			const lang = CODE_LANGS[path.extname(entry.name).slice(1).toLowerCase()];
			if (!lang || (rel === '' && entry.name === 'README.md')) continue;
			chapter.files.push({ rel: childRel, lang, slug: `${chapter.slug}/${slugify(childRel)}` });
			found = true;
		}
	}
	return found;
}

function partDirs() {
	return numbered(REPO_DIR, /^part(\d+)-/);
}

/** 이름이 pattern에 맞는 하위 디렉터리를 번호 순서로 돌려줍니다(ch2가 ch10보다 앞). */
function numbered(absDir, pattern) {
	return fs
		.readdirSync(absDir, { withFileTypes: true })
		.filter((entry) => entry.isDirectory() && pattern.test(entry.name))
		.map((entry) => entry.name)
		.sort((a, b) => Number(a.match(pattern)[1]) - Number(b.match(pattern)[1]));
}

// ---------------------------------------------------------------------------
// 마크다운 변환

/** 코드 블록 밖의 줄에만 fn(line, 줄 번호)을 적용합니다. */
function mapProse(markdown, fn) {
	let fence = null;
	return markdown
		.split('\n')
		.map((line, i) => {
			const marker = line.match(/^\s*(`{3,}|~{3,})/)?.[1];
			if (fence) {
				if (marker && marker[0] === fence[0] && marker.length >= fence.length && line.trim() === marker) fence = null;
				return line;
			}
			if (marker) {
				fence = marker;
				return line;
			}
			return fn(line, i);
		})
		.join('\n');
}

/** 첫 번째 H1을 제목으로 떼어 냅니다. Starlight가 title을 H1으로 그리므로 본문에서는 지웁니다. */
function splitTitle(markdown, fallback) {
	let index = -1;
	mapProse(markdown, (line, i) => {
		if (index < 0 && /^#\s+\S/.test(line)) index = i;
		return line;
	});
	if (index < 0) return { title: fallback, body: markdown };

	const lines = markdown.split('\n');
	const [heading] = lines.splice(index, 1);
	const title = heading.replace(/^#\s+/, '').replace(/\s+#*\s*$/, '').replace(/[`*]/g, '');
	return { title, body: lines.join('\n').replace(/^\n+/, '') };
}

/** 인라인 코드 밖에 있는 `](대상)` 형태의 링크 대상을 resolve로 바꿉니다. */
function rewriteLinks(markdown, resolve) {
	const link = /\]\(([^()\s]+)((?:\s+"[^"]*")?)\)/g;
	const codeSpan = /(`+)(?:(?!\1)[^])*?\1(?!`)/g;
	return mapProse(markdown, (line) => {
		let result = '';
		let last = 0;
		const replace = (text) => text.replace(link, (_, target, title) => `](${resolve(target)}${title})`);
		for (const match of line.matchAll(codeSpan)) {
			result += replace(line.slice(last, match.index)) + match[0];
			last = match.index + match[0].length;
		}
		return result + replace(line.slice(last));
	});
}

function longestBacktickRun(text) {
	return Math.max(0, ...(text.match(/`+/g) ?? []).map((run) => run.length));
}

/** Astro가 파일 경로에서 만드는 slug와 같아지도록 경로를 정리합니다. */
function slugify(rel) {
	return rel
		.split('/')
		.map((segment) => segment.toLowerCase().replace(/[^a-z0-9_-]+/g, '-'))
		.join('/');
}

function slugToFile(slug, isIndex) {
	if (slug === '') return 'index.md';
	return isIndex ? `${slug}/index.md` : `${slug}.md`;
}

function page(sourceRel, frontmatter, body) {
	return ['---', `# 자동 생성 파일입니다. 원본: ${sourceRel}`, ...frontmatter, '---', '', body].join('\n');
}

// ---------------------------------------------------------------------------
// 쓰기

/** 내용이 바뀐 파일만 쓰고, 더 이상 만들지 않는 파일은 지웁니다. */
function writeOutput(output) {
	let written = 0;
	let removed = 0;

	for (const [rel, content] of output) {
		const abs = path.join(OUT_DIR, rel);
		if (fs.existsSync(abs) && fs.readFileSync(abs, 'utf8') === content) continue;
		fs.mkdirSync(path.dirname(abs), { recursive: true });
		fs.writeFileSync(abs, content);
		written++;
	}

	const prune = (absDir, rel) => {
		if (!fs.existsSync(absDir)) return;
		for (const entry of fs.readdirSync(absDir, { withFileTypes: true })) {
			const childRel = rel ? `${rel}/${entry.name}` : entry.name;
			const childAbs = path.join(absDir, entry.name);
			if (entry.isDirectory()) {
				prune(childAbs, childRel);
				if (fs.readdirSync(childAbs).length === 0) fs.rmdirSync(childAbs);
			} else if (!output.has(childRel)) {
				fs.rmSync(childAbs);
				removed++;
			}
		}
	};
	prune(OUT_DIR, '');

	return { written, removed };
}

// `node scripts/sync-content.mjs` 로 직접 실행할 때
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
	const { siteOptions } = await import('../site.config.mjs');
	const { written, removed } = syncContent(siteOptions);
	console.log(`[sync-content] ${written}개 갱신, ${removed}개 삭제`);
}
