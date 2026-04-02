import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'architecture/overview',
        'architecture/domain-model',
        'architecture/parsing-pipeline',
      ],
    },
    {
      type: 'category',
      label: 'Parsing',
      items: [
        'parsing/variables',
        'parsing/secrets',
        'parsing/config-merging',
        'parsing/validation',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api-reference/teckel-model',
        'api-reference/teckel-parser',
        'api-reference/error-catalog',
      ],
    },
  ],
};

export default sidebars;
