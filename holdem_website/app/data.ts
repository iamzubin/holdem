type Project = {
  name: string
  description: string
  link: string
  media: string
  mediaType?: 'image' | 'video'
  id: string
}

type WorkExperience = {
  company: string
  title: string
  start: string
  end: string
  link: string
  id: string
}

type BlogPost = {
  title: string
  description: string
  link: string
  uid: string
}

type SocialLink = {
  label: string
  link: string
}

export const PROJECTS: Project[] = [
  {
    name: 'Holdem',
    description:
      'Holdem is an open-source tool for smoother file drag-and-drop management, offering a temporary holding area for files and folders. ',
    link: 'https://github.com/iamzubin/holdem',
    media:
      '/assets/holdem.gif',
    mediaType: 'image',
    id: 'project1',
  },
  {
    name: 'FFreed',
    description: 'Simplifying Ethereum account management with ERC-4337 and Smart Contract Wallets. Easy Web3 integration, gas sponsorship, and seamless user onboarding..',
    link: 'https://x.com/ffreedwallet',
    media:
    '/assets/ffreed-image.jpg',
    mediaType: 'image',
    id: 'project2',
  },
  {
    name: 'Stubs',
    description: 'A creative page-builder that empowers creators to craft personalized websites with Web3 elements, like NFT showcases and blockchain integration.',
    link: 'https://github.com/iamzubin',
    media:
    '/assets/stubs.mp4',
    mediaType: 'video',
    id: 'project3',
  },
  {
    name: 'Chain Speak',
    description: 'Conversational AI for seamless interactions with the Polygon ecosystem. Perform token transfers, swaps, staking, and cross-chain bridging effortlessly, making DeFi accessible to all.',
    link: 'https://github.com/iamzubin',
    media:
    '/assets/chain-speak.mp4',
    mediaType: 'video',
    id: 'project4',
  },
]

export const WORK_EXPERIENCE: WorkExperience[] = [
  {
    company: 'FFREED',
    title: 'Founder',
    start: 'May 2024',
    end: 'Present',
    link: 'https://x.com/ffreedwallet/',
    id: 'work1',
  },
  {
    company: 'Freelance + Open Source',
    title: 'Software Developer',
    start: '2024',
    end: 'Present',
    link: 'https://github.com/iamzubin',
    id: 'work2',
  },
  {
    company: 'Stubs',
    title: 'Smart Contract + Frontend Developer',
    start: '2022',
    end: '2023',
    link: 'https://x.com/StubsWorldHQ',
    id: 'work3',
  },
  {
    company: 'Sublime Finance',
    title: 'Frontend + Solidity Developer',
    start: '2021',
    end: '2022',
    link: 'https://x.com/sublimefinance',
    id: 'work4',
  },
  {
    company: 'Google Summer of Code (Fedora)',
    title: 'Open Source Developer',
    start: '2019',
    end: '2019',
    link: 'https://communityblog.fedoraproject.org/gsoc-summer-2019-fedora-gooey-karma/',
    id: 'work5',
  },
]

export const BLOG_POSTS: BlogPost[] = [
  {
    title: 'Built dev tools for deploying contracts without gas or keys',
    description: 'Integrated OAuth-based private key schemes',
    link: '/blog/exploring-the-intersection-of-design-ai-and-design-engineering',
    uid: 'blog-1',
  },
  {
    title: 'Built cross-chain tools using PolygonZkEVM, EVM bridges',
    description: 'Worked on security hooks for Hyperlane',
    link: '/blog/exploring-the-intersection-of-design-ai-and-design-engineering',
    uid: 'blog-2',
  },
  {
    title: 'Deployed ERC721/1155 contracts, built governance + relayers',
    description: 'Built React token grid UI for on-chain collections',
    link: '/blog/exploring-the-intersection-of-design-ai-and-design-engineering',
    uid: 'blog-3',
  },
]

export const SOCIAL_LINKS: SocialLink[] = [
  {
    label: 'Cal',
    link: 'https://cal.com/iamzubin',
  },
  {
    label: 'GitHub',
    link: 'https://github.com/iamzubin',
  },
  {
    label: 'Twitter',
    link: 'https://twitter.com/iamzub_in',
  },
  {
    label: 'LinkedIn',
    link: 'https://linkedin.com/in/iamzubin',
  },
]

export const EMAIL = 'me@iamzub.in'
