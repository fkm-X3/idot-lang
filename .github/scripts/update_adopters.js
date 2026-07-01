const fs = require('fs');
const path = require('path');

// Paths to files
const jsonPath = path.join(__dirname, '../../data/early_adopters.json');
const mdPath = path.join(__dirname, '../../EARLY_ADOPTERS.md');

// Read JSON data
const adopters = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));

// Cap at 200 slots
const activeAdopters = adopters.slice(0, 200);

// Generate Markdown Header
let markdownContent = `# Language Early Adopters Hall of Fame\n\n`;
markdownContent += `These are the first 200 unique repositories helping to get our language officially recognized by GitHub.\n\n`;
markdownContent += `| Slot | Developer | Project & Link | Description |\n`;
markdownContent += `| :--- | :--- | :--- | :--- |\n`;

// Populate Table Rows dynamically
activeAdopters.forEach((adopter, index) => {
  const slotNum = String(index + 1).padStart(3, '0');
  const repoName = adopter.repo_url.split('/').pop();
  
  markdownContent += `| **#${slotNum}** | [@${adopter.github_username}](https://github.com/${adopter.github_username}) | [${repoName}](${adopter.repo_url}) | ${adopter.project_description} |\n`;
});

// Write to EARLY_ADOPTERS.md
fs.writeFileSync(mdPath, markdownContent, 'utf8');
console.log('EARLY_ADOPTERS.md successfully regenerated!');