//
//  FPGameAtlas.m
//  IronJump
//
//  Created by Filip Kunc on 6/21/10.
//  For license see LICENSE.TXT
//

#import "FPGameAtlas.h"

FPGameAtlas *gameAtlas = nil;

@implementation FPGameAtlas

@synthesize texture, verticesUsed;

+ (FPGameAtlas *)sharedAtlas
{
	if (!gameAtlas)
		gameAtlas = [[FPGameAtlas alloc] initWithFile:@"levelatlas.png"];
	return gameAtlas;
}

- (id)initWithFile:(NSString *)fileName
{
	self = [super init];
	if (self)
	{
		texture = [[FPTexture alloc] initWithFile:fileName convertToAlpha:NO];
		vertexPtr = globalVertexBuffer;
		verticesUsed = 0;
		
		texCoordNormalizeX = 1.0f / [texture width];
		texCoordNormalizeY = 1.0f / [texture height];
	}
	return self;
}

- (void)removeAllTiles
{
	vertexPtr = globalVertexBuffer;
	verticesUsed = 0;
}

- (void)drawAllTiles
{
	if (verticesUsed > 0)
	{
		glEnable(GL_TEXTURE_2D);
		glBindTexture(GL_TEXTURE_2D, [texture textureID]);
		glEnableClientState(GL_VERTEX_ARRAY);
		glVertexPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), &globalVertexBuffer->x);	
		glEnableClientState(GL_TEXTURE_COORD_ARRAY);
		glTexCoordPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), &globalVertexBuffer->s);		
		glDrawArrays(GL_TRIANGLES, 0, verticesUsed);
	}
	vertexPtr = globalVertexBuffer;
	verticesUsed = 0;
}

- (void)addTile:(CGRect)aTile atPoint:(CGPoint)aPoint
{
	if (aPoint.x > 400.0f || aPoint.x + aTile.size.width < -80.0f ||
		aPoint.y > 400.0f || aPoint.y + aTile.size.height < -80.0f)
		return;
	
	const float left = CGRectGetMinX(aTile) * texCoordNormalizeX;
	const float right = CGRectGetMaxX(aTile) * texCoordNormalizeX;
	const float top = CGRectGetMinY(aTile) * texCoordNormalizeY;
	const float bottom = CGRectGetMaxY(aTile) * texCoordNormalizeY;
	
	const FPAtlasVertex vertices[] = 
	{
		{ aPoint.x,						aPoint.y,						left,	top,	},	// 0
		{ aPoint.x + aTile.size.width,	aPoint.y,						right,	top,	},	// 1
		{ aPoint.x,						aPoint.y + aTile.size.height,	left,	bottom, },	// 2
		
		{ aPoint.x,						aPoint.y + aTile.size.height,	left,	bottom, },	// 2
		{ aPoint.x + aTile.size.width,	aPoint.y + aTile.size.height,	right,	bottom, },	// 3
		{ aPoint.x + aTile.size.width,	aPoint.y,						right,	top,	},	// 1
	};
	
	verticesUsed += 6;
	if (verticesUsed > kMaxVertices)
	{
		@throw [NSException exceptionWithName:@"addTile failed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
	}
	
	memcpy(vertexPtr, vertices, sizeof(vertices));
	vertexPtr += 6;
}

- (void)addTile:(CGRect)aTile atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	const float left = CGRectGetMinX(aTile) * texCoordNormalizeX;
	const float right = CGRectGetMaxX(aTile) * texCoordNormalizeX;
	const float top = CGRectGetMinY(aTile) * texCoordNormalizeY;
	const float bottom = CGRectGetMaxY(aTile) * texCoordNormalizeY;
	
	for (int y = 0; y < aHeightSegments; y++)
	{
		for (int x = 0; x < aWidthSegments; x++)
		{
			CGPoint pt = CGPointMake(aPoint.x + x * aTile.size.width, aPoint.y + y * aTile.size.height);
			
			if (pt.x > 400.0f || pt.x + aTile.size.width < -80.0f ||
				pt.y > 400.0f || pt.y + aTile.size.height < -80.0f)
				continue;				
			
			const FPAtlasVertex vertices[] = 
			{
				{ pt.x,						pt.y,						left,	top,	},	// 0
				{ pt.x + aTile.size.width,	pt.y,						right,	top,	},	// 1
				{ pt.x,						pt.y + aTile.size.height,	left,	bottom, },	// 2
				
				{ pt.x,						pt.y + aTile.size.height,	left,	bottom, },	// 2
				{ pt.x + aTile.size.width,	pt.y + aTile.size.height,	right,	bottom, },	// 3
				{ pt.x + aTile.size.width,	pt.y,						right,	top,	},	// 1
			};
			
			verticesUsed += 6;
			if (verticesUsed > kMaxVertices)
			{
				@throw [NSException exceptionWithName:@"addTile failed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
			}
			
			memcpy(vertexPtr, vertices, sizeof(vertices));
			vertexPtr += 6;			
		}
	}
}

- (void)addTileWithoutCheck:(CGRect)aTile atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	const float left = CGRectGetMinX(aTile) * texCoordNormalizeX;
	const float right = CGRectGetMaxX(aTile) * texCoordNormalizeX;
	const float top = CGRectGetMinY(aTile) * texCoordNormalizeY;
	const float bottom = CGRectGetMaxY(aTile) * texCoordNormalizeY;
	
	for (int y = 0; y < aHeightSegments; y++)
	{
		for (int x = 0; x < aWidthSegments; x++)
		{
			CGPoint pt = CGPointMake(aPoint.x + x * aTile.size.width, aPoint.y + y * aTile.size.height);		
			
			const FPAtlasVertex vertices[] = 
			{
				{ pt.x,						pt.y,						left,	top,	},	// 0
				{ pt.x + aTile.size.width,	pt.y,						right,	top,	},	// 1
				{ pt.x,						pt.y + aTile.size.height,	left,	bottom, },	// 2
				
				{ pt.x,						pt.y + aTile.size.height,	left,	bottom, },	// 2
				{ pt.x + aTile.size.width,	pt.y + aTile.size.height,	right,	bottom, },	// 3
				{ pt.x + aTile.size.width,	pt.y,						right,	top,	},	// 1
			};
			
			verticesUsed += 6;
			if (verticesUsed > kMaxVertices)
			{
				@throw [NSException exceptionWithName:@"addTile failed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
			}
			
			memcpy(vertexPtr, vertices, sizeof(vertices));
			vertexPtr += 6;			
		}
	}
}

- (void)addPlayerAtPoint:(CGPoint)aPoint rotation:(float)aRotation
{
	CGRect aTile = CGRectMake(1.0f, 1.0f, 32.0f, 32.0f);
	
	const float left = CGRectGetMinX(aTile) * texCoordNormalizeX;
	const float right = CGRectGetMaxX(aTile) * texCoordNormalizeX;
	const float top = CGRectGetMinY(aTile) * texCoordNormalizeY;
	const float bottom = CGRectGetMaxY(aTile) * texCoordNormalizeY;
	
	FPAtlasVertex vertices[] = 
	{
		{ aPoint.x,						aPoint.y,						left,	top,	},	// 0
		{ aPoint.x + aTile.size.width,	aPoint.y,						right,	top,	},	// 1
		{ aPoint.x,						aPoint.y + aTile.size.height,	left,	bottom, },	// 2
		
		{ aPoint.x,						aPoint.y + aTile.size.height,	left,	bottom, },	// 2
		{ aPoint.x + aTile.size.width,	aPoint.y + aTile.size.height,	right,	bottom, },	// 3
		{ aPoint.x + aTile.size.width,	aPoint.y,						right,	top,	},	// 1
	};
	
	float centerX = aPoint.x + 16.0f;
	float centerY = aPoint.y + 16.0f;
	aRotation = (aRotation / 180.0f) * M_PI;
	float sinRotation = sinf(aRotation);
	float cosRotation = cosf(aRotation);
	
	for (int i = 0; i < 6; i++)
	{
		float x = vertices[i].x - centerX;
		float y = vertices[i].y - centerY;
		vertices[i].x = centerX + x * cosRotation - y * sinRotation;
		vertices[i].y = centerY + x * sinRotation + y * cosRotation;
	}
	
	verticesUsed += 6;
	if (verticesUsed > kMaxVertices)
	{
		@throw [NSException exceptionWithName:@"addPlayer failed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
	}
	
	memcpy(vertexPtr, vertices, sizeof(vertices));
	vertexPtr += 6;	
}

- (void)addDiamondAtPoint:(CGPoint)aPoint
{
	CGRect tile = CGRectMake(34.0f, 1.0f, 32.0f, 32.0f);
	[self addTile:tile atPoint:aPoint];
}

- (void)addBackgroundWithIndex:(int)index widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	CGPoint aPoint = CGPointZero;
    CGRect aTile;
    if (index == 0)
        aTile = CGRectMake(67.0f, 1.0f, 32.0f, 32.0f);
    else
        aTile = CGRectMake(34.0f, 34.0f, 32.0f, 32.0f);
	
	[self addTileWithoutCheck:aTile atPoint:aPoint widthSegments:aWidthSegments heightSegments:aHeightSegments];
}

- (void)addMovableAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	CGRect tile = CGRectMake(100.0f, 1.0f, 32.0f, 32.0f);
	[self addTile:tile atPoint:aPoint widthSegments:aWidthSegments heightSegments:aHeightSegments];
}

- (void)addPlatformAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	CGRect tile = CGRectMake(133.0f, 1.0f, 32.0f, 32.0f);
	[self addTile:tile atPoint:aPoint widthSegments:aWidthSegments heightSegments:aHeightSegments];
}

- (void)addTrampoline:(int)index atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	CGRect tile;
	switch (index)
	{
		case 0: tile = CGRectMake(166.0f, 1.0f, 64.0f, 32.0f); break;
		case 1: tile = CGRectMake(231.0f, 1.0f, 64.0f, 32.0f); break;
		case 2: tile = CGRectMake(296.0f, 1.0f, 64.0f, 32.0f); break;
        default: return;
	}
	[self addTile:tile atPoint:aPoint widthSegments:aWidthSegments heightSegments:aHeightSegments];
}

- (void)addElevator:(int)index atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	CGRect tile;
	switch (index)
	{
		case 0: tile = CGRectMake(361.0f, 1.0f, 32.0f, 32.0f); break;
		case 1: tile = CGRectMake(394.0f, 1.0f, 32.0f, 32.0f); break;
		case 2: tile = CGRectMake(427.0f, 1.0f, 32.0f, 32.0f); break;
        default: return;
	}
	[self addTile:tile atPoint:aPoint widthSegments:aWidthSegments heightSegments:aHeightSegments];
}

- (void)addMagnetAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments
{
	CGRect tile = CGRectMake(460.0f, 1.0f, 32.0f, 32.0f);
	[self addTile:tile atPoint:aPoint widthSegments:aWidthSegments heightSegments:1];
}

- (void)addSpeedPowerUpAtPoint:(CGPoint)aPoint
{
	CGRect tile = CGRectMake(1.0f, 34.0f, 32.0f, 32.0f);
	[self addTile:tile atPoint:aPoint widthSegments:1 heightSegments:1];
}

- (void)addExitAtPoint:(CGPoint)aPoint
{
	CGRect tile = CGRectMake(67.0f, 34.0f, 64.0f, 64.0f);
	[self addTile:tile atPoint:aPoint];
}

- (void)addSpeedEffectAtPoint:(CGPoint)aPoint
{
	CGRect tile = CGRectMake(132.0f, 34.0f, 64.0f, 64.0f);
	[self addTile:tile atPoint:aPoint widthSegments:1 heightSegments:1];
}

@end
