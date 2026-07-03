'use client';

import './code-highlight-block.css';

import type { Options } from 'react-markdown';

import rehypeRaw from 'rehype-raw';
import remarkGfm from 'remark-gfm';
import { useId, useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import { mergeClasses, isExternalLink } from 'minimal-shared/utils';

import Link from '@mui/material/Link';

import { RouterLink } from 'src/routes/components';

import { Image } from '../image';
import { MarkdownRoot } from './styles';
import { markdownClasses } from './classes';
import { htmlToMarkdown, isMarkdownContent } from './html-to-markdown';

// ----------------------------------------------------------------------

export type MarkdownProps = React.ComponentProps<typeof MarkdownRoot> & Options;

export function Markdown({
  sx,
  children,
  className,
  components,
  rehypePlugins,
  ...other
}: MarkdownProps) {
  const content = useMemo(() => {
    const cleanedContent = String(children).trim();

    return isMarkdownContent(cleanedContent) ? cleanedContent : htmlToMarkdown(cleanedContent);
  }, [children]);

  const allRehypePlugins = useMemo(
    () => [...defaultRehypePlugins, ...(rehypePlugins ?? [])],
    [rehypePlugins]
  );

  return (
    <MarkdownRoot className={mergeClasses([markdownClasses.root, className])} sx={sx}>
      <ReactMarkdown
        components={{ ...defaultComponents, ...components }}
        rehypePlugins={allRehypePlugins}
        /* base64-encoded images
         * https://github.com/remarkjs/react-markdown/issues/774
         * urlTransform={(value: string) => value}
         */
        {...other}
      >
        {content}
      </ReactMarkdown>
    </MarkdownRoot>
  );
}

/** **************************************
 * @rehypePlugins
 *************************************** */
const defaultRehypePlugins: NonNullable<Options['rehypePlugins']> = [
  rehypeRaw,
  rehypeHighlight,
  [remarkGfm, { singleTilde: false }],
];

/** **************************************
 * @components
 * Note: node is passed by react-markdown, but we intentionally omit or rename it
 * (e.g., node: _n) to prevent rendering it as [object Object] in the DOM.
 *************************************** */
const defaultComponents: NonNullable<Options['components']> = {
  img: ({ node: _n, onLoad: _o, ...other }) => (
    <Image
      ratio="16/9"
      className={markdownClasses.content.image}
      sx={{ borderRadius: 2 }}
      {...other}
    />
  ),
  a: ({ href = '', children, node: _n, ...other }) => {
    const linkProps = isExternalLink(href)
      ? { target: '_blank', rel: 'noopener noreferrer' }
      : { component: RouterLink };

    return (
      <Link {...linkProps} href={href} className={markdownClasses.content.link} {...other}>
        {children}
      </Link>
    );
  },
  pre: ({ children }) => (
    <div className={markdownClasses.content.codeBlock}>
      <pre>{children}</pre>
    </div>
  ),
  code: ({ className = '', children, node: _n, ...other }) => {
    const hasLanguage = /language-\w+/.test(className);
    const appliedClass = hasLanguage ? className : markdownClasses.content.codeInline;

    return (
      <code className={appliedClass} {...other}>
        {children}
      </code>
    );
  },
  input: ({ type, node: _n, ...other }) =>
    type === 'checkbox' ? (
      <CustomCheckbox className={markdownClasses.content.checkbox} {...other} />
    ) : (
      <input type={type} {...other} />
    ),
};

function CustomCheckbox(props: React.ComponentProps<'input'>) {
  const uniqueId = useId();
  return <input type="checkbox" id={uniqueId} {...props} />;
}
