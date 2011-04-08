//
//  FPGame.m
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  For license see LICENSE.TXT
//

#import "FPGame.h"
#import "FPTexture.h"
#import "FPFont.h"
#import "FPDiamond.h"

FPFont *font = nil;
FPTexture *background = nil;
GLuint vertexBufferID = 0;
int oldBackgroundIndex = 1;
int backgroundIndex = 1;
const int widthSegments = 17;
const int heightSegments = 14;

void CreateVertexBuffer() 
{
    [[FPGameAtlas sharedAtlas] removeAllTiles];
	[[FPGameAtlas sharedAtlas] addBackgroundWithIndex:backgroundIndex widthSegments:widthSegments heightSegments:heightSegments];
	glGenBuffers(1, &vertexBufferID);
	glBindBuffer(GL_ARRAY_BUFFER, vertexBufferID); 
	glBufferData(GL_ARRAY_BUFFER, widthSegments * heightSegments * 6 * sizeof(FPAtlasVertex), globalVertexBuffer, GL_STATIC_DRAW);
    oldBackgroundIndex = backgroundIndex;
}

void DrawUsingVertexBuffer()
{
	glBindBuffer(GL_ARRAY_BUFFER, vertexBufferID); 
	glEnableClientState(GL_VERTEX_ARRAY); 
	glVertexPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), (void*)offsetof(FPAtlasVertex, x)); 
	glEnableClientState(GL_TEXTURE_COORD_ARRAY); 
	glTexCoordPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), (void*)offsetof(FPAtlasVertex, s)); 
	glDrawArrays(GL_TRIANGLES, 0, widthSegments * heightSegments * 6);
}

void UnbindVertexBuffer()
{
	glBindBuffer(GL_ARRAY_BUFFER, 0);
}

void ChangeVertexBufferIfNeeded()
{
    if (backgroundIndex != oldBackgroundIndex)
    {
        glBindBuffer(GL_ARRAY_BUFFER, 0);
        [[FPGameAtlas sharedAtlas] removeAllTiles];
        [[FPGameAtlas sharedAtlas] addBackgroundWithIndex:backgroundIndex widthSegments:widthSegments heightSegments:heightSegments];
        glBindBuffer(GL_ARRAY_BUFFER, vertexBufferID); 
        glBufferData(GL_ARRAY_BUFFER, widthSegments * heightSegments * 6 * sizeof(FPAtlasVertex), globalVertexBuffer, GL_STATIC_DRAW);
        oldBackgroundIndex = backgroundIndex;
    }
}

@implementation FPGame

@synthesize inputAcceleration, width, height, player, gameObjects, backgroundOffset;

+ (void)loadFontAndBackgroundIfNeeded
{
	if (!font)
		font = [[FPFont alloc] initWithFile:@"AndaleMono.png" tileSize:32 spacing:13.0f];
#if TARGET_OS_IPHONE
	if (!vertexBufferID)
	{
		CreateVertexBuffer();
		UnbindVertexBuffer();
	}
#else
	if (!background)
		background = [[FPTexture alloc] initWithFile:@"marbleblue.png" convertToAlpha:NO];
#endif
}

+ (FPFont *)font
{
	return font;
}

+ (void)setBackgroundIndex:(int)index
{
    backgroundIndex = index;
}

- (id)initWithWidth:(float)aWidth height:(float)aHeight
{
	self = [super init];
	if (self)
	{
		gameObjects = [[NSMutableArray alloc] init];
		player = [[FPPlayer alloc] initWithWidth:aWidth height:aHeight];
		
		inputAcceleration = CGPointZero;
		width = aWidth;
		height = aHeight;
		backgroundOffset = CGPointZero;
		
		diamondsPicked = 0;
		diamondsCount = 0;

#ifdef MEASURE_FPS
		lastDate = [NSDate date];
		[lastDate retain];
		
		fpsCounter = 0;
		currentFPS = @"FPS";
#endif		

	}
	return self;
}

- (void)dealloc
{
	[gameObjects release];
	[player release];
	[super dealloc];
}

- (id)initWithBinaryData:(NSData *)data width:(float)aWidth height:(float)aHeight
{
	self = [self initWithWidth:aWidth height:aHeight];
	if (self)
	{
		id unarchived = [NSKeyedUnarchiver unarchiveObjectWithData:data];
		if (unarchived)
		{
			NSArray *unarchivedArray = (NSArray *)unarchived;
			[gameObjects removeAllObjects];
            lastPlayerX = 0.0f;
            lastPlayerY = 0.0f;
			for (NSUInteger i = 0; i < [unarchivedArray count]; i++)
			{
				id object = [unarchivedArray objectAtIndex:i];
				if ([object isMemberOfClass:[FPPlayer class]])
				{
					FPPlayer *lastPlayer = (FPPlayer *)object;
                    lastPlayerX = lastPlayer.x;
                    lastPlayerY = lastPlayer.y;
				}
				else 
				{
					if ([object isMemberOfClass:[FPDiamond class]])
						diamondsCount++;
					[gameObjects addObject:object];
				}
			}
			float moveX = player.x - lastPlayerX;
			float moveY = player.y - lastPlayerY;
			[self moveWorldWithX:moveX y:moveY];
		}
	}
	return self;
}

- (id)initWithXMLData:(NSData *)data width:(float)aWidth height:(float)aHeight
{
    self = [self initWithWidth:aWidth height:aHeight];
    if (self)
    {
        [gameObjects removeAllObjects];
        lastPlayerX = 0.0f;
        lastPlayerY = 0.0f;
        
        NSXMLParser *parser = [[NSXMLParser alloc] initWithData:data];
        FPXMLParser *parserHelper = [[FPXMLParser alloc] init];
        [parserHelper setDelegate:self];    
        [parser setDelegate:parserHelper];
        
        BOOL succeded = [parser parse];
        if (!succeded)
            NSLog(@"parserError: %@", [parser parserError]);
        
        [parser release];
        [parserHelper release];
        
        if (!succeded)
            return nil;
        
        float moveX = player.x - lastPlayerX;
        float moveY = player.y - lastPlayerY;
        [self moveWorldWithX:moveX y:moveY];
    }
    return self;
}

- (void)parser:(FPXMLParser *)parser foundObject:(id<FPGameObject>)gameObject
{
    if ([gameObject isMemberOfClass:[FPPlayer class]])
    {
        FPPlayer *lastPlayer = (FPPlayer *)gameObject;
        lastPlayerX = lastPlayer.x;
        lastPlayerY = lastPlayer.y;
    }
    else 
    {
        if ([gameObject isMemberOfClass:[FPDiamond class]])
            diamondsCount++;
        [gameObjects addObject:gameObject];
    }
}

- (void)update
{
    diamondsPicked = 0;
    
	for (id<FPGameObject> gameObject in gameObjects)
	{
        if (!gameObject.isVisible)
        {
            if ([gameObject isMemberOfClass:[FPDiamond class]])
                diamondsPicked++;
        }
        
		if (!gameObject.isMovable)
			[gameObject updateWithGame:self];
	}
    
    for (id<FPGameObject> gameObject in gameObjects)
	{
        if (gameObject.isMovable)
			[gameObject updateWithGame:self];
	}
	
	[player updateWithGame:self];
}

- (void)draw
{
	[FPGame loadFontAndBackgroundIfNeeded];
	CGPoint offset = CGPointMake(fmodf(backgroundOffset.x, 32.0f) - 32.0f,
								 fmodf(backgroundOffset.y, 32.0f) - 32.0f);
	
	glDisable(GL_BLEND);    

#if TARGET_OS_IPHONE
    ChangeVertexBufferIfNeeded();
    
	offset.x -= 80.0f;
	offset.y += 80.0f;
	glEnable(GL_TEXTURE_2D);
	glBindTexture(GL_TEXTURE_2D, [[FPGameAtlas sharedAtlas] texture].textureID);
	glPushMatrix();
	glTranslatef(offset.x, offset.y, 0);
	DrawUsingVertexBuffer();
	glPopMatrix();
	
	UnbindVertexBuffer();
	
	[[FPGameAtlas sharedAtlas] removeAllTiles];
#else
	[background drawAtPoint:offset widthSegments:width / background.width + 3 heightSegments:height / background.height + 2];
#endif
	
	glDisable(GL_BLEND);
	for (id<FPGameObject> gameObject in gameObjects)
	{
        if (!gameObject.isVisible)
            continue;
        
		if (!gameObject.isTransparent)
			[gameObject draw];
	}
	
#if TARGET_OS_IPHONE	
	[[FPGameAtlas sharedAtlas] drawAllTiles];
#endif
	
	glEnable(GL_BLEND);
	for (id<FPGameObject> gameObject in gameObjects)
	{
        if (!gameObject.isVisible)
            continue;
        
		if (gameObject.isTransparent)
			[gameObject draw];
	}
	[player draw];

#if TARGET_OS_IPHONE	
	[[FPGameAtlas sharedAtlas] drawAllTiles];	
#endif
	
	[player drawSpeedUp];
	
	glColor4f(1, 1, 1, 0.8f);
	
#ifdef MEASURE_FPS
	fpsCounter++;
	NSDate *now = [NSDate date];
	NSTimeInterval timeInterval = [now timeIntervalSinceDate:lastDate];
	if (timeInterval > 1.0)
	{
		[lastDate release];
		lastDate = now;
		[lastDate retain];
		[currentFPS release];
		currentFPS = [[NSString alloc] initWithFormat:@"FPS: %.2f", fpsCounter / timeInterval];
		fpsCounter = 0;
	}
	
	[font drawText:currentFPS atPoint:CGPointZero];
	
#else
	NSString *diamondsText = [NSString stringWithFormat:@"Diamonds: %i/%i", diamondsPicked, diamondsCount];
	NSString *speedUpText = player.speedUpCounter == 0 ? nil : [NSString stringWithFormat:@"%.1f", (maxSpeedUpCount - player.speedUpCounter) / 60.0f]; 
	
	[font drawText:diamondsText atPoint:CGPointMake(3.0f, 0.0f)];
	glColor4f(0.5f, 1.0f, 1.0f, 0.8f);
	[font drawText:speedUpText atPoint:CGPointMake(430.0f, 285.0f)];
		
#endif
	
	glColor4f(1, 1, 1, 1);
}

- (void)moveWorldWithX:(float)x y:(float)y
{
	for (id<FPGameObject> gameObject in gameObjects)
	{
		[gameObject moveWithX:x y:y];
	}
	
	backgroundOffset.x += x * 0.25f;
	backgroundOffset.y += y * 0.25f;
}

@end
