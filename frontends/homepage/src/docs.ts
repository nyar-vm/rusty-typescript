export interface DocNode {
    id: string;
    title: string;
    path: string;
    isDirectory: boolean;
    children?: DocNode[];
    parentId?: string;
    order?: number;
}

export interface DocMetadata {
    title?: string;
    order?: number;
}

import { docsModules as rawDocsModules } from "virtual:docs";

const docsModules: Record<string, string> = {};
for (const key in rawDocsModules) {
    const normalizedKey = key.replace(/\\/g, "/");
    docsModules[`../../documentation/zh-hans/${normalizedKey}`] = rawDocsModules[key];
}

console.log("Loaded docsModules:", Object.keys(docsModules));

function parseDocMetadata(content: string): DocMetadata {
    const metadata: DocMetadata = {};
    const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);

    if (frontmatterMatch) {
        const frontmatter = frontmatterMatch[1];
        const titleMatch = frontmatter.match(/^title:\s*(.+)$/m);
        const orderMatch = frontmatter.match(/^order:\s*(\d+)$/m);

        if (titleMatch) {
            metadata.title = titleMatch[1].trim().replace(/^['"]|['"]$/g, "");
        }
        if (orderMatch) {
            metadata.order = parseInt(orderMatch[1], 10);
        }
    }

    return metadata;
}

function normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
}

function generateId(path: string): string {
    const normalizedPath = normalizePath(path);
    return normalizedPath
        .replace("../../documentation/zh-hans/", "")
        .replace(/\.md$/, "")
        .replace(/\//g, "-");
}

function getDocTitle(path: string, metadata: DocMetadata): string {
    if (metadata.title) {
        return metadata.title;
    }

    const normalizedPath = normalizePath(path);
    const filename = normalizedPath.split("/").pop()?.replace(".md", "") || "";

    const titleMap: Record<string, string> = {
        index: "首页",
        introduction: "介绍",
        "quick-start": "快速开始",
        features: "功能特性",
        agent: "智能体",
        capabilities: "能力",
        chat: "对话",
        memory: "记忆",
        scheduler: "调度器",
        skills: "技能",
        tool: "工具",
        workspace: "工作区",
        extensibility: "可扩展性",
        performance: "性能",
        security: "安全性",
        "agent-core": "智能体核心",
        architecture: "架构",
        "core-layer": "核心层",
        decentralization: "去中心化",
        "ecosystem-overview": "生态系统概览",
        infrastructure: "基础设施",
        "master-plan": "总体规划",
        "protocol-layer": "协议层",
        "data-models": "数据模型",
        "technology-choices": "技术选择",
        skynet: "天网",
        messages: "消息",
        profile: "配置",
        resources: "资源",
        subnets: "子网",
        "threat-model": "威胁模型",
        uri: "URI",
        "add-skills": "添加技能",
        "configure-agent": "配置智能体",
        "getting-started": "开始使用",
        "use-tools": "使用工具",
        "best-practices": "最佳实践",
        "development-helper": "开发助手",
        "knowledge-base": "知识库",
        "personal-assistant": "个人助手",
        "task-automation": "任务自动化",
        readme: "说明",
        advanced: "高级",
        concepts: "概念",
        maintainer: "维护者",
        overview: "概览",
        tutorials: "教程",
        "use-cases": "使用案例",
    };

    return titleMap[filename] || filename;
}

function buildDocTree(docs: DocNode[]): DocNode[] {
    const nodeMap: Record<string, DocNode> = {};
    const pathToNode: Record<string, DocNode> = {};

    // 首先创建所有文档节点
    docs.forEach((doc) => {
        const normalizedPath = normalizePath(doc.path);
        const node = { ...doc, path: normalizedPath, children: [] };
        nodeMap[doc.id] = node;
        pathToNode[normalizedPath] = node;
    });

    // 创建三个主要分类
    const categories: Record<string, DocNode> = {
        beginner: {
            id: "beginner",
            title: "入门",
            path: "beginner",
            isDirectory: true,
            children: [],
        },
        advanced: {
            id: "advanced",
            title: "进阶",
            path: "advanced",
            isDirectory: true,
            children: [],
        },
        development: {
            id: "development",
            title: "开发",
            path: "development",
            isDirectory: true,
            children: [],
        },
    };

    const root: DocNode[] = Object.values(categories);

    // 文档分类映射
    const docCategoryMap: Record<string, string> = {
        // 入门
        "guide-introduction": "beginner",
        "guide-concepts-index": "beginner",
        "guide-concepts-orm": "beginner",
        "guide-concepts-rpc": "beginner",
        "guide-concepts-xrpc": "beginner",

        // 进阶
        "guide-architecture": "advanced",
        "guide-xrpc-core": "advanced",
        "language-index": "advanced",
        "language-basics": "advanced",
        "language-entity-modeling": "advanced",
        "language-query-basics": "advanced",
        "language-relations": "advanced",
        "language-aggregation": "advanced",
        "language-namespaces": "advanced",
        "language-traits": "advanced",
        "language-unions": "advanced",
        "language-micro-functions": "advanced",
        "language-projection": "advanced",
        "language-modern-paradigms": "advanced",
        "language-engineering": "advanced",
        "language-philosophy": "advanced",
        "language-security": "advanced",
        "language-udf-wasi": "advanced",

        // 开发
        "guide-commands-index": "development",
        "guide-commands-backup": "development",
        "guide-commands-check": "development",
        "guide-commands-compile": "development",
        "guide-commands-deployment": "development",
        "guide-commands-generate": "development",
        "guide-commands-migration": "development",
        "guide-commands-pull": "development",
        "maintenance-index": "development",
        "maintenance-internals": "development",
        "maintenance-tokio-integration": "development",
        "maintenance-database-index": "development",
        "maintenance-database-mysql": "development",
        "maintenance-database-pgsql": "development",
        "maintenance-database-redis": "development",
        "maintenance-database-sqlite": "development",
        "guide-roadmap": "development",
    };

    // 将文档添加到对应的分类
    docs.forEach((doc) => {
        const category = docCategoryMap[doc.id];
        if (category && categories[category]) {
            categories[category].children?.push(doc);
        } else {
            // 未分类的文档添加到根目录
            root.push(doc);
        }
    });

    const sortNodes = (nodes: DocNode[]): DocNode[] => {
        return nodes
            .sort((a, b) => {
                // 目录优先
                if (a.isDirectory && !b.isDirectory) return -1;
                if (!a.isDirectory && b.isDirectory) return 1;
                // 按顺序排序
                return (a.order || 999) - (b.order || 999);
            })
            .map((node) => ({
                ...node,
                children: node.children ? sortNodes(node.children) : undefined,
            }));
    };

    return sortNodes(root);
}

export async function loadDocs(): Promise<DocNode[]> {
    console.log("docsModules keys:", Object.keys(docsModules));
    const docs: DocNode[] = [];

    for (const path in docsModules) {
        const content = docsModules[path] as string;
        const metadata = parseDocMetadata(content);
        const id = generateId(path);
        const title = getDocTitle(path, metadata);

        docs.push({
            id,
            title,
            path,
            isDirectory: false,
            order: metadata.order,
        });
    }

    console.log("Loaded docs:", docs);
    const tree = buildDocTree(docs);
    console.log("Built doc tree:", tree);
    return tree;
}

export async function getDocContent(path: string): Promise<string> {
    const normalizedPath = normalizePath(path);
    return (docsModules[normalizedPath] as string) || (docsModules[path] as string) || "";
}
