---
name: pet
description: Manage your desktop pet — open, close, set character, toggle auto-start, install character packs
user-invocable: true
---

# /pet command

Manage the Claude Status Pet desktop companion.

## Usage

The user will run `/pet` with an optional subcommand. Parse the arguments and execute accordingly:

### Subcommands

- `/pet` or `/pet open` — Open pet(s) for all active sessions
- `/pet close` — Close all running pets  
- `/pet set <character>` — Switch character. Available: `ferris`, `mona`, `kuromi`, `chonk`, `cat`, `ghost`, `robot`, `duck`, `snail`, `axolotl`, or any installed custom pack name
- `/pet auto on` — Enable auto-start on new sessions
- `/pet auto off` — Disable auto-start
- `/pet status` — Show current config, active sessions, and installed character packs
- `/pet update` — Update binary and assets to latest release
- `/pet pack install <url-or-path>` — Install a custom character pack
- `/pet pack list` — List all installed character packs
- `/pet pack remove <name>` — Remove a custom character pack
- `/pet pack create <name>` — Create a character pack template
- `/pet help` — Show available commands

## Implementation

> **All commands use `node` for cross-platform compatibility** (Node.js is guaranteed in all environments: Claude Code, Copilot, etc.). Do NOT use bash-only or PowerShell-only commands.

Key paths (use `os.homedir()` in node, or `~` in bash):
- **Pet data dir**: `~/.claude/pet-data/`
- **Pet binary**: `~/.claude/pet-data/bin/claude-status-pet*`
- **Status files**: `~/.claude/pet-data/status-*.json` (one per session)
- **Config file**: `~/.claude/pet-data/config.json`
- **Assets dir**: `~/.claude/pet-data/assets/` (DLC characters: mona, kuromi)
- **Custom packs dir**: `~/.claude/pet-data/characters/` (user-installed packs)

### Basic commands

For `open`: launch the pet binary for each status file:
```js
node -e "
const fs=require('fs'),path=require('path'),{execSync,spawn}=require('child_process'),os=require('os');
const dir=path.join(os.homedir(),'.claude','pet-data');
const bin=fs.readdirSync(path.join(dir,'bin')).find(f=>f.startsWith('claude-status-pet'));
if(!bin){console.log('Pet binary not found');process.exit(1)}
const binPath=path.join(dir,'bin',bin);
const assets=path.join(dir,'assets');
fs.readdirSync(dir).filter(f=>f.startsWith('status-')&&f.endsWith('.json')).forEach(f=>{
  const sid=f.replace('status-','').replace('.json','');
  const args=['--status-file',path.join(dir,f),'--session-id',sid];
  if(fs.existsSync(assets))args.push('--assets-dir',assets);
  spawn(binPath,args,{detached:true,stdio:'ignore'}).unref();
});
console.log('Pet(s) launched');
"
```

For `close`: kill all pet processes:
```js
node -e "
const{execSync}=require('child_process'),os=require('os');
if(os.platform()==='win32'){try{execSync('tasklist /NH',{encoding:'utf8'}).split('\\n').filter(l=>l.includes('claude-status-pet')).forEach(l=>{const pid=l.trim().split(/\\s+/)[1];execSync('taskkill /F /PID '+pid)});console.log('Closed')}catch(e){console.log('No pets running')}}
else{try{execSync('pkill -f claude-status-pet');console.log('Closed')}catch(e){console.log('No pets running')}}
"
```

For `set <char>`: update config.json:
```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const cfg=path.join(os.homedir(),'.claude','pet-data','config.json');
let c={};try{c=JSON.parse(fs.readFileSync(cfg,'utf8'))}catch(e){}
c.character='<CHAR>';
fs.writeFileSync(cfg,JSON.stringify(c,null,2));
console.log('Default character set to: <CHAR>');
"
```

For `auto on/off`: update config.json field `auto_start`:
```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const cfg=path.join(os.homedir(),'.claude','pet-data','config.json');
let c={};try{c=JSON.parse(fs.readFileSync(cfg,'utf8'))}catch(e){}
c.auto_start=<true|false>;
fs.writeFileSync(cfg,JSON.stringify(c,null,2));
console.log('Auto-start: <on|off>');
"
```

For `status`: show config and installed packs:
```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const dir=path.join(os.homedir(),'.claude','pet-data');
const cfg=path.join(dir,'config.json');
let c={};try{c=JSON.parse(fs.readFileSync(cfg,'utf8'))}catch(e){}
console.log('Config:',JSON.stringify(c,null,2));
const sessions=fs.readdirSync(dir).filter(f=>f.startsWith('status-')&&f.endsWith('.json'));
console.log('Active sessions:',sessions.length);
sessions.forEach(f=>console.log('  '+f));
for(const sub of ['assets','characters']){
  const d=path.join(dir,sub);
  if(!fs.existsSync(d))continue;
  fs.readdirSync(d).filter(f=>fs.existsSync(path.join(d,f,'character.json'))).forEach(f=>{
    const cfg2=JSON.parse(fs.readFileSync(path.join(d,f,'character.json'),'utf8'));
    console.log((sub==='assets'?'DLC':'Custom')+': '+cfg2.name+' ('+f+')');
  });
}
"
```

### Character Pack commands

#### `/pet pack list`

```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const dir=path.join(os.homedir(),'.claude','pet-data');
for(const[label,sub]of[['DLC','assets'],['Custom','characters']]){
  const d=path.join(dir,sub);
  if(!fs.existsSync(d))continue;
  const packs=fs.readdirSync(d).filter(f=>fs.existsSync(path.join(d,f,'character.json')));
  if(packs.length){console.log('=== '+label+' ===');packs.forEach(f=>{
    const c=JSON.parse(fs.readFileSync(path.join(d,f,'character.json'),'utf8'));
    console.log('  '+c.name+' ('+c.type+', '+Object.keys(c.states).length+' states)');
  })}
}
"
```

#### `/pet pack install <url-or-path>`

Install a custom character pack from a URL (zip) or local directory:

**From URL:**
```js
node -e "
const https=require('https'),fs=require('fs'),path=require('path'),os=require('os'),{execSync}=require('child_process');
const url='<USER_URL>';
const charsDir=path.join(os.homedir(),'.claude','pet-data','characters');
fs.mkdirSync(charsDir,{recursive:true});
const tmp=path.join(os.tmpdir(),'pet-pack.zip');
const file=fs.createWriteStream(tmp);
const get=(u)=>https.get(u,r=>{if(r.statusCode>=300&&r.headers.location)get(r.headers.location);else r.pipe(file).on('finish',()=>{
  file.close();
  if(os.platform()==='win32')execSync('powershell -Command \"Expand-Archive -Path \\\"'+tmp+'\\\" -DestinationPath \\\"'+charsDir+'\\\" -Force\"');
  else execSync('unzip -o \"'+tmp+'\" -d \"'+charsDir+'\"');
  fs.unlinkSync(tmp);
  console.log('Pack installed to '+charsDir);
})});
get(url);
"
```

**From local path:** copy directory to `~/.claude/pet-data/characters/`:
```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const src='<LOCAL_PATH>';
const name=path.basename(src);
const dest=path.join(os.homedir(),'.claude','pet-data','characters',name);
fs.cpSync(src,dest,{recursive:true});
const c=JSON.parse(fs.readFileSync(path.join(dest,'character.json'),'utf8'));
console.log('Installed: '+c.name+' ('+Object.keys(c.states).length+' states)');
"
```

Tell the user: "Character pack installed! Right-click the pet → look under **Custom** to select it."

#### `/pet pack remove <name>`

```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const dir=path.join(os.homedir(),'.claude','pet-data','characters','<NAME>');
if(fs.existsSync(dir)){fs.rmSync(dir,{recursive:true});console.log('Removed: <NAME>')}
else{console.log('Pack not found: <NAME>')}
"
```

#### `/pet pack create <name>`

Create a character pack template:

```js
node -e "
const fs=require('fs'),path=require('path'),os=require('os');
const name='<NAME>';
const dir=path.join(os.homedir(),'.claude','pet-data','characters',name);
fs.mkdirSync(dir,{recursive:true});
fs.writeFileSync(path.join(dir,'character.json'),JSON.stringify({
  name:name,type:'gif',
  states:{idle:[name+'/idle.gif'],thinking:[name+'/thinking.gif'],reading:[name+'/reading.gif'],editing:[name+'/editing.gif'],searching:[name+'/searching.gif'],running:[name+'/running.gif'],delegating:[name+'/delegating.gif'],waiting:[name+'/waiting.gif'],error:[name+'/error.gif'],offline:[name+'/offline.gif'],unknown:[name+'/idle.gif']}
},null,2));
console.log('Template created at: '+dir);
console.log('Next: add images, edit character.json, restart pet');
"
```

Always give a short confirmation after executing.
